//! Refine Loop — boucle de raffinement autorecursive.
//!
//! Itère sur prompt-engine jusqu'à convergence qualité.

use prompt_engine::ScoredPrompt;
use thiserror::Error;

/// Erreurs refine loop.
#[derive(Debug, Error)]
pub enum RefineError {
    #[error("max iterations reached without convergence")]
    MaxIterations,
    #[error("prompt error: {0}")]
    Prompt(#[from] prompt_engine::PromptError),
}

pub type RefineResult<T> = Result<T, RefineError>;

/// Configuration de la boucle de raffinement.
pub struct RefineConfig {
    pub max_iterations: usize,
    pub target_score: f64,
}

impl Default for RefineConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            target_score: 0.9,
        }
    }
}

/// Vérifie si un prompt a atteint le seuil de qualité.
pub fn is_converged(prompt: &ScoredPrompt, config: &RefineConfig) -> bool {
    prompt.score >= config.target_score
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
}
