//! Prompt Engine — AZR self-play refinement.
//!
//! Génère, évalue et affine les prompts de manière autorecursive.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Erreurs prompt engine.
#[derive(Debug, Error)]
pub enum PromptError {
    #[error("refinement failed: {0}")]
    Refinement(String),
}

pub type PromptResult<T> = Result<T, PromptError>;

/// Prompt avec score de qualité.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredPrompt {
    pub text: String,
    pub score: f64,
    pub iteration: usize,
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
}
