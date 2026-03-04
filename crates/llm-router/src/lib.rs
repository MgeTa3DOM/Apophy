//! LLM Router — routage souverain entre modèles GGUF locaux.
//!
//! Sélectionne le modèle optimal selon la tâche (CPU/GPU, tokens, type).
//! Modèles Apophy : FunctionGemma:270m (CPU), GLM-4.7-Flash (GPU), Gemma3:270m (CPU).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

/// Erreurs routeur.
#[derive(Debug, Error)]
pub enum RouterError {
    #[error("no model available for task: {0}")]
    NoModel(String),
    #[error("inference failed: {0}")]
    Inference(String),
    #[error("model not found: {0}")]
    NotFound(String),
}

pub type RouterResult<T> = Result<T, RouterError>;

/// Ressource cible pour un modèle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resource {
    Cpu,
    Gpu,
}

/// Type de tâche pour le routage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    Orchestration,
    Vision,
    CodeGen,
    Embedding,
}

/// Descripteur de modèle local.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDescriptor {
    pub name: String,
    pub role: String,
    pub resource: Resource,
    pub max_tokens: usize,
    pub supported_tasks: Vec<TaskType>,
    pub gpu_memory_mb: Option<u32>,
}

/// Trait pour backend d'inférence (llama.cpp, etc.).
#[async_trait]
pub trait InferenceBackend: Send + Sync {
    async fn generate(&self, model: &str, prompt: &str, max_tokens: usize) -> RouterResult<String>;
    async fn is_healthy(&self, model: &str) -> bool;
}

/// Registre de modèles avec routage par tâche.
pub struct ModelRegistry {
    models: Vec<ModelDescriptor>,
}

impl ModelRegistry {
    /// Crée un registre vide.
    pub fn new() -> Self {
        Self { models: Vec::new() }
    }

    /// Crée le registre Apophy par défaut (3 modèles de CLAUDE.md).
    pub fn apophy_default() -> Self {
        let mut reg = Self::new();
        reg.register(ModelDescriptor {
            name: "functiongemma-270m".to_string(),
            role: "orchestration".to_string(),
            resource: Resource::Cpu,
            max_tokens: 2048,
            supported_tasks: vec![TaskType::Orchestration],
            gpu_memory_mb: None,
        });
        reg.register(ModelDescriptor {
            name: "glm-4.7-flash".to_string(),
            role: "vision-code".to_string(),
            resource: Resource::Gpu,
            max_tokens: 8192,
            supported_tasks: vec![TaskType::Vision, TaskType::CodeGen],
            gpu_memory_mb: Some(4096),
        });
        reg.register(ModelDescriptor {
            name: "gemma3-270m".to_string(),
            role: "embedding".to_string(),
            resource: Resource::Cpu,
            max_tokens: 2048,
            supported_tasks: vec![TaskType::Embedding],
            gpu_memory_mb: None,
        });
        reg
    }

    /// Enregistre un modèle.
    pub fn register(&mut self, model: ModelDescriptor) {
        info!(model = %model.name, "registered");
        self.models.push(model);
    }

    /// Trouve le meilleur modèle pour un type de tâche.
    pub fn route(&self, task: TaskType) -> RouterResult<&ModelDescriptor> {
        self.models
            .iter()
            .find(|m| m.supported_tasks.contains(&task))
            .ok_or_else(|| RouterError::NoModel(format!("{task:?}")))
    }

    /// Trouve un modèle par nom.
    pub fn get(&self, name: &str) -> RouterResult<&ModelDescriptor> {
        self.models
            .iter()
            .find(|m| m.name == name)
            .ok_or_else(|| RouterError::NotFound(name.to_string()))
    }

    /// Liste tous les modèles CPU (fallback si GPU indisponible).
    pub fn cpu_models(&self) -> Vec<&ModelDescriptor> {
        self.models.iter().filter(|m| m.resource == Resource::Cpu).collect()
    }

    /// Liste tous les modèles enregistrés.
    pub fn all(&self) -> &[ModelDescriptor] {
        &self.models
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
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
            supported_tasks: vec![TaskType::Embedding],
            gpu_memory_mb: None,
        };
        let json = serde_json::to_string(&desc).expect("serialize failed");
        let back: ModelDescriptor = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(back.name, "gemma3-270m");
        assert_eq!(back.resource, Resource::Cpu);
    }

    #[test]
    fn apophy_default_registry() {
        let reg = ModelRegistry::apophy_default();
        assert_eq!(reg.all().len(), 3);
    }

    #[test]
    fn route_by_task() {
        let reg = ModelRegistry::apophy_default();

        let orch = reg.route(TaskType::Orchestration).expect("no orchestration model");
        assert_eq!(orch.name, "functiongemma-270m");

        let vision = reg.route(TaskType::Vision).expect("no vision model");
        assert_eq!(vision.name, "glm-4.7-flash");

        let embed = reg.route(TaskType::Embedding).expect("no embedding model");
        assert_eq!(embed.name, "gemma3-270m");
    }

    #[test]
    fn cpu_fallback() {
        let reg = ModelRegistry::apophy_default();
        let cpu = reg.cpu_models();
        assert_eq!(cpu.len(), 2);
        assert!(cpu.iter().all(|m| m.resource == Resource::Cpu));
    }

    #[test]
    fn get_by_name() {
        let reg = ModelRegistry::apophy_default();
        let model = reg.get("glm-4.7-flash").expect("not found");
        assert_eq!(model.resource, Resource::Gpu);
        assert_eq!(model.gpu_memory_mb, Some(4096));

        assert!(reg.get("nonexistent").is_err());
    }
}
