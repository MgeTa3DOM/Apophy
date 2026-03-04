//! LLM Router — routage souverain entre modèles GGUF locaux.
//!
//! Sélectionne le modèle optimal selon la tâche (CPU/GPU, tokens, type).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Erreurs routeur.
#[derive(Debug, Error)]
pub enum RouterError {
    #[error("no model available for task: {0}")]
    NoModel(String),
    #[error("inference failed: {0}")]
    Inference(String),
}

pub type RouterResult<T> = Result<T, RouterError>;

/// Ressource cible pour un modèle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resource {
    Cpu,
    Gpu,
}

/// Descripteur de modèle local.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDescriptor {
    pub name: String,
    pub role: String,
    pub resource: Resource,
    pub max_tokens: usize,
}

/// Trait pour backend d'inférence (llama.cpp, etc.).
#[async_trait]
pub trait InferenceBackend: Send + Sync {
    async fn generate(&self, model: &str, prompt: &str, max_tokens: usize) -> RouterResult<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_descriptor_serializes() {
        let desc = ModelDescriptor {
            name: "gemma3-270m".to_string(),
            role: "embedding".to_string(),
            resource: Resource::Cpu,
            max_tokens: 2048,
        };
        let json = serde_json::to_string(&desc).expect("serialize failed");
        let back: ModelDescriptor = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(back.name, "gemma3-270m");
        assert_eq!(back.resource, Resource::Cpu);
    }
}
