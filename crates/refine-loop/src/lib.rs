//! Refine Loop — boucle de raffinement autorecursive AZR.
//!
//! Itère sur prompt-engine jusqu'à convergence qualité.
//! Pattern : generate → evaluate → mutate → re-evaluate → converge.

use prompt_engine::{
    evaluate_prompt, mutate_prompt, EvalCriteria, MutationType, PromptError, RefinementHistory,
    ScoredPrompt,
};
use thiserror::Error;
use tracing::{info, warn};

/// Erreurs refine loop.
#[derive(Debug, Error)]
pub enum RefineError {
    #[error("max iterations reached without convergence (best: {best_score:.3})")]
    MaxIterations { best_score: f64 },
    #[error("prompt error: {0}")]
    Prompt(#[from] PromptError),
}

pub type RefineResult<T> = Result<T, RefineError>;

/// Configuration de la boucle de raffinement.
#[derive(Debug, Clone)]
pub struct RefineConfig {
    pub max_iterations: usize,
    pub target_score: f64,
    pub criteria: EvalCriteria,
    /// Stratégies de mutation à essayer dans l'ordre.
    pub mutations: Vec<MutationType>,
}

impl Default for RefineConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            target_score: 0.9,
            criteria: EvalCriteria::default(),
            mutations: vec![
                MutationType::Rephrase,
                MutationType::Expand,
                MutationType::Compress,
                MutationType::Constrain,
                MutationType::Reorder,
            ],
        }
    }
}

/// Vérifie si un prompt a atteint le seuil de qualité.
pub fn is_converged(prompt: &ScoredPrompt, config: &RefineConfig) -> bool {
    prompt.score >= config.target_score
}

/// Exécute la boucle de raffinement complète.
///
/// Itère : evaluate → pick best mutation → re-evaluate → until converged.
pub fn refine(initial_text: &str, config: &RefineConfig) -> RefineResult<RefinementHistory> {
    let mut history = RefinementHistory::new("refine-session", config.criteria.clone());

    // Score initial
    let initial_score = evaluate_prompt(initial_text, &config.criteria)?;
    let mut current = ScoredPrompt {
        text: initial_text.to_string(),
        score: initial_score,
        iteration: 0,
    };
    history.record(current.clone());

    info!(score = initial_score, "initial prompt scored");

    if is_converged(&current, config) {
        info!("converged immediately at iteration 0");
        return Ok(history);
    }

    for i in 1..=config.max_iterations {
        // Essayer chaque mutation et garder la meilleure
        let mut best_variant = current.clone();

        for mutation in &config.mutations {
            let variant = mutate_prompt(&current.text, *mutation);
            let score = evaluate_prompt(&variant.mutated, &config.criteria)?;

            if score > best_variant.score {
                best_variant = ScoredPrompt {
                    text: variant.mutated,
                    score,
                    iteration: i,
                };
            }
        }

        // Si aucune amélioration, on garde l'ancien mais on continue
        if best_variant.score <= current.score {
            warn!(iteration = i, "no improvement found, continuing");
            let stale = ScoredPrompt {
                text: current.text.clone(),
                score: current.score,
                iteration: i,
            };
            history.record(stale);
        } else {
            info!(
                iteration = i,
                score = best_variant.score,
                delta = best_variant.score - current.score,
                "improved"
            );
            history.record(best_variant.clone());
            current = best_variant;
        }

        if is_converged(&current, config) {
            info!(iteration = i, score = current.score, "converged");
            return Ok(history);
        }
    }

    Err(RefineError::MaxIterations {
        best_score: history.best.as_ref().map(|b| b.score).unwrap_or(0.0),
    })
}

/// Version single-step : applique une mutation et retourne le résultat scoré.
pub fn refine_once(
    text: &str,
    mutation: MutationType,
    criteria: &EvalCriteria,
) -> RefineResult<ScoredPrompt> {
    let variant = mutate_prompt(text, mutation);
    let score = evaluate_prompt(&variant.mutated, criteria)?;
    Ok(ScoredPrompt {
        text: variant.mutated,
        score,
        iteration: 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convergence_check() {
        let config = RefineConfig::default();

        let good = ScoredPrompt {
            text: "test".to_string(),
            score: 0.95,
            iteration: 1,
        };
        assert!(is_converged(&good, &config));

        let bad = ScoredPrompt {
            text: "test".to_string(),
            score: 0.5,
            iteration: 1,
        };
        assert!(!is_converged(&bad, &config));
    }

    #[test]
    fn refine_loop_produces_history() {
        let config = RefineConfig {
            max_iterations: 5,
            target_score: 0.99, // intentionally unreachable
            ..Default::default()
        };

        let result = refine("Explain topology", &config);
        // Either converges or hits max iterations — both produce useful data
        match result {
            Ok(history) => {
                assert!(!history.iterations.is_empty());
                assert!(history.best.is_some());
            }
            Err(RefineError::MaxIterations { best_score }) => {
                assert!(best_score > 0.0);
            }
            Err(e) => panic!("unexpected error: {e}"),
        }
    }

    #[test]
    fn refine_once_returns_scored() {
        let criteria = EvalCriteria::default();
        let result = refine_once("test prompt", MutationType::Rephrase, &criteria)
            .expect("refine_once failed");
        assert!(result.score > 0.0);
        assert!(result.text.contains("Clearly explain"));
    }

    #[test]
    fn refine_with_easy_target() {
        let config = RefineConfig {
            max_iterations: 5,
            target_score: 0.5, // easy to reach
            ..Default::default()
        };

        let history = refine("Explain: the concept of algebraic topology", &config)
            .expect("should converge");
        assert!(!history.iterations.is_empty());
        assert!(history.best.as_ref().map(|b| b.score).unwrap_or(0.0) >= 0.5);
    }

    #[test]
    fn default_config_has_all_mutations() {
        let config = RefineConfig::default();
        assert_eq!(config.mutations.len(), 5);
    }
}
