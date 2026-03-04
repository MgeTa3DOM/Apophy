//! Prompt Engine — AZR self-play refinement.
//!
//! Génère, évalue et affine les prompts de manière autorecursive.
//! AZR = AlphaZero-style Reasoning : le modèle s'évalue lui-même.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Erreurs prompt engine.
#[derive(Debug, Error)]
pub enum PromptError {
    #[error("refinement failed: {0}")]
    Refinement(String),
    #[error("evaluation failed: {0}")]
    Evaluation(String),
    #[error("generation failed: {0}")]
    Generation(String),
}

pub type PromptResult<T> = Result<T, PromptError>;

/// Prompt avec score de qualité AZR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredPrompt {
    pub text: String,
    pub score: f64,
    pub iteration: usize,
}

/// Variante générée lors du self-play.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVariant {
    pub original: String,
    pub mutated: String,
    pub mutation_type: MutationType,
}

/// Types de mutations AZR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationType {
    /// Rephrase pour clarté.
    Rephrase,
    /// Ajoute du contexte.
    Expand,
    /// Réduit le bruit.
    Compress,
    /// Restructure l'ordre logique.
    Reorder,
    /// Ajoute des contraintes.
    Constrain,
}

/// Critères d'évaluation AZR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalCriteria {
    /// Poids pour la clarté (0..1).
    pub clarity: f64,
    /// Poids pour la spécificité (0..1).
    pub specificity: f64,
    /// Poids pour la concision (0..1).
    pub conciseness: f64,
    /// Poids pour la complétude (0..1).
    pub completeness: f64,
}

impl Default for EvalCriteria {
    fn default() -> Self {
        Self {
            clarity: 0.3,
            specificity: 0.3,
            conciseness: 0.2,
            completeness: 0.2,
        }
    }
}

impl EvalCriteria {
    /// Vérifie que les poids somment à ~1.0.
    pub fn validate(&self) -> PromptResult<()> {
        let sum = self.clarity + self.specificity + self.conciseness + self.completeness;
        if (sum - 1.0).abs() > 0.01 {
            return Err(PromptError::Evaluation(
                format!("criteria weights must sum to 1.0, got {sum:.3}"),
            ));
        }
        Ok(())
    }
}

/// Historique d'une session de raffinement AZR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinementHistory {
    pub session_id: String,
    pub iterations: Vec<ScoredPrompt>,
    pub best: Option<ScoredPrompt>,
    pub criteria: EvalCriteria,
}

impl RefinementHistory {
    /// Crée un nouvel historique.
    pub fn new(session_id: impl Into<String>, criteria: EvalCriteria) -> Self {
        Self {
            session_id: session_id.into(),
            iterations: Vec::new(),
            best: None,
            criteria,
        }
    }

    /// Enregistre une itération et met à jour le best.
    pub fn record(&mut self, prompt: ScoredPrompt) {
        let is_best = self.best.as_ref().map_or(true, |b| prompt.score > b.score);
        if is_best {
            self.best = Some(prompt.clone());
        }
        self.iterations.push(prompt);
    }

    /// Score moyen sur toutes les itérations.
    pub fn avg_score(&self) -> f64 {
        if self.iterations.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.iterations.iter().map(|p| p.score).sum();
        sum / self.iterations.len() as f64
    }

    /// Tendance de progression (delta entre dernier et premier).
    pub fn trend(&self) -> f64 {
        if self.iterations.len() < 2 {
            return 0.0;
        }
        let first = self.iterations.first().map(|p| p.score).unwrap_or(0.0);
        let last = self.iterations.last().map(|p| p.score).unwrap_or(0.0);
        last - first
    }
}

/// Applique une mutation textuelle simple (heuristique, pas LLM).
pub fn mutate_prompt(text: &str, mutation: MutationType) -> PromptVariant {
    let mutated = match mutation {
        MutationType::Rephrase => format!("Clearly explain: {text}"),
        MutationType::Expand => format!("{text}\nProvide detailed reasoning and examples."),
        MutationType::Compress => {
            // Garde les 80% premiers tokens (heuristique)
            let words: Vec<&str> = text.split_whitespace().collect();
            let keep = (words.len() as f64 * 0.8).ceil() as usize;
            words[..keep.min(words.len())].join(" ")
        }
        MutationType::Reorder => {
            let sentences: Vec<&str> = text.split(". ").collect();
            if sentences.len() > 1 {
                let mut reordered = sentences.clone();
                reordered.reverse();
                reordered.join(". ")
            } else {
                text.to_string()
            }
        }
        MutationType::Constrain => {
            format!("{text}\nConstraints: be precise, avoid ambiguity, max 3 sentences.")
        }
    };

    PromptVariant {
        original: text.to_string(),
        mutated,
        mutation_type: mutation,
    }
}

/// Évalue un prompt selon les critères AZR (scoring heuristique local).
///
/// Retourne un score 0..1. En production, ceci appelle le LLM via llm-router.
pub fn evaluate_prompt(text: &str, criteria: &EvalCriteria) -> PromptResult<f64> {
    criteria.validate()?;

    let words: Vec<&str> = text.split_whitespace().collect();
    let word_count = words.len();
    let sentence_count = text.matches(". ").count() + 1;

    // Heuristiques simples (remplacées par LLM scoring en Phase 3)
    let clarity_score = if word_count > 5 && word_count < 200 { 0.8 } else { 0.4 };
    let specificity_score = if text.contains(':') || text.contains("explain") { 0.7 } else { 0.5 };
    let conciseness_score = if word_count < 50 { 0.9 } else { 0.5 };
    let completeness_score = if sentence_count >= 2 { 0.8 } else { 0.6 };

    let score = criteria.clarity * clarity_score
        + criteria.specificity * specificity_score
        + criteria.conciseness * conciseness_score
        + criteria.completeness * completeness_score;

    Ok(score.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scored_prompt_roundtrip() {
        let prompt = ScoredPrompt {
            text: "Explain quantum entanglement".to_string(),
            score: 0.85,
            iteration: 3,
        };
        let json = serde_json::to_string(&prompt).expect("serialize failed");
        let back: ScoredPrompt = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(back.iteration, 3);
        assert!((back.score - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn eval_criteria_validation() {
        let valid = EvalCriteria::default();
        assert!(valid.validate().is_ok());

        let invalid = EvalCriteria {
            clarity: 0.5,
            specificity: 0.5,
            conciseness: 0.5,
            completeness: 0.5,
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn evaluate_prompt_scoring() {
        let criteria = EvalCriteria::default();
        let score = evaluate_prompt("Explain the concept of: topological data analysis", &criteria)
            .expect("eval failed");
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn mutation_rephrase() {
        let variant = mutate_prompt("test prompt", MutationType::Rephrase);
        assert!(variant.mutated.contains("Clearly explain"));
        assert_eq!(variant.mutation_type, MutationType::Rephrase);
    }

    #[test]
    fn mutation_compress() {
        let long = "word ".repeat(20);
        let variant = mutate_prompt(&long, MutationType::Compress);
        let original_words = long.split_whitespace().count();
        let mutated_words = variant.mutated.split_whitespace().count();
        assert!(mutated_words <= original_words);
    }

    #[test]
    fn mutation_expand() {
        let variant = mutate_prompt("test", MutationType::Expand);
        assert!(variant.mutated.contains("detailed reasoning"));
    }

    #[test]
    fn mutation_constrain() {
        let variant = mutate_prompt("test", MutationType::Constrain);
        assert!(variant.mutated.contains("Constraints"));
    }

    #[test]
    fn refinement_history_tracking() {
        let criteria = EvalCriteria::default();
        let mut history = RefinementHistory::new("session-1", criteria);

        history.record(ScoredPrompt {
            text: "v1".to_string(),
            score: 0.5,
            iteration: 0,
        });
        history.record(ScoredPrompt {
            text: "v2".to_string(),
            score: 0.8,
            iteration: 1,
        });
        history.record(ScoredPrompt {
            text: "v3".to_string(),
            score: 0.7,
            iteration: 2,
        });

        assert_eq!(history.iterations.len(), 3);
        assert_eq!(history.best.as_ref().map(|b| &b.text), Some(&"v2".to_string()));
        assert!((history.avg_score() - 0.6666).abs() < 0.01);
        assert!((history.trend() - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn empty_history_defaults() {
        let history = RefinementHistory::new("empty", EvalCriteria::default());
        assert_eq!(history.avg_score(), 0.0);
        assert_eq!(history.trend(), 0.0);
        assert!(history.best.is_none());
    }
}
