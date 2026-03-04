//! LLM Router — intelligent model selection for local inference.
//!
//! Routes inference requests to the optimal model based on task type,
//! model capabilities, and resource availability. Backend-agnostic:
//! works with any GGUF-compatible runtime (llama.cpp, vLLM, etc.).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

/// Router errors.
#[derive(Debug, Error)]
pub enum RouterError {
    #[error("no model available for task: {0}")]
    NoModel(String),
    #[error("inference failed: {0}")]
    Inference(String),
    #[error("model not found: {0}")]
    NotFound(String),
    #[error("backend unavailable: {0}")]
    BackendUnavailable(String),
}

pub type RouterResult<T> = Result<T, RouterError>;

/// Compute resource type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resource {
    Cpu,
    Gpu,
}

/// Task type for routing decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    Orchestration,
    Vision,
    CodeGen,
    Embedding,
    Chat,
}

/// Model descriptor — backend-agnostic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDescriptor {
    pub name: String,
    pub role: String,
    pub resource: Resource,
    pub max_tokens: usize,
    pub supported_tasks: Vec<TaskType>,
    /// Estimated memory footprint in MB (for capacity planning).
    pub memory_mb: Option<u32>,
}

/// Trait for pluggable inference backends.
///
/// Implement this for llama.cpp, vLLM, Ollama, or any local runtime.
#[async_trait]
pub trait InferenceBackend: Send + Sync {
    /// Generate a completion from the model.
    async fn generate(&self, model: &str, prompt: &str, max_tokens: usize) -> RouterResult<String>;
    /// Check if the backend is healthy and ready.
    async fn is_healthy(&self, model: &str) -> bool;
}

/// Model registry with task-based routing.
pub struct ModelRegistry {
    models: Vec<ModelDescriptor>,
}

impl ModelRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
        }
    }

    /// Creates a default registry with common small models.
    ///
    /// Override with your own models via `register()`.
    pub fn with_defaults() -> Self {
        let mut reg = Self::new();
        reg.register(ModelDescriptor {
            name: "router-small".to_string(),
            role: "orchestration".to_string(),
            resource: Resource::Cpu,
            max_tokens: 2048,
            supported_tasks: vec![TaskType::Orchestration],
            memory_mb: Some(512),
        });
        reg.register(ModelDescriptor {
            name: "coder-medium".to_string(),
            role: "code-generation".to_string(),
            resource: Resource::Cpu,
            max_tokens: 8192,
            supported_tasks: vec![TaskType::CodeGen, TaskType::Chat],
            memory_mb: Some(4096),
        });
        reg.register(ModelDescriptor {
            name: "embedder-small".to_string(),
            role: "embedding".to_string(),
            resource: Resource::Cpu,
            max_tokens: 2048,
            supported_tasks: vec![TaskType::Embedding],
            memory_mb: Some(256),
        });
        reg
    }

    /// Registers a model in the registry.
    pub fn register(&mut self, model: ModelDescriptor) {
        info!(model = %model.name, role = %model.role, "model registered");
        self.models.push(model);
    }

    /// Routes a task to the best available model.
    pub fn route(&self, task: TaskType) -> RouterResult<&ModelDescriptor> {
        self.models
            .iter()
            .find(|m| m.supported_tasks.contains(&task))
            .ok_or_else(|| RouterError::NoModel(format!("{task:?}")))
    }

    /// Routes with a resource preference (CPU or GPU).
    pub fn route_with_resource(
        &self,
        task: TaskType,
        resource: Resource,
    ) -> RouterResult<&ModelDescriptor> {
        // Try preferred resource first
        if let Some(model) = self
            .models
            .iter()
            .find(|m| m.supported_tasks.contains(&task) && m.resource == resource)
        {
            return Ok(model);
        }
        // Fallback to any resource
        self.route(task)
    }

    /// Finds a model by name.
    pub fn get(&self, name: &str) -> RouterResult<&ModelDescriptor> {
        self.models
            .iter()
            .find(|m| m.name == name)
            .ok_or_else(|| RouterError::NotFound(name.to_string()))
    }

    /// Lists models filtered by resource type.
    pub fn by_resource(&self, resource: Resource) -> Vec<&ModelDescriptor> {
        self.models
            .iter()
            .filter(|m| m.resource == resource)
            .collect()
    }

    /// Returns all registered models.
    pub fn all(&self) -> &[ModelDescriptor] {
        &self.models
    }

    /// Total estimated memory footprint in MB.
    pub fn total_memory_mb(&self) -> u32 {
        self.models
            .iter()
            .filter_map(|m| m.memory_mb)
            .sum()
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
            name: "test-model".to_string(),
            role: "embedding".to_string(),
            resource: Resource::Cpu,
            max_tokens: 2048,
            supported_tasks: vec![TaskType::Embedding],
            memory_mb: Some(256),
        };
        let json = serde_json::to_string(&desc).expect("serialize failed");
        let back: ModelDescriptor = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(back.name, "test-model");
        assert_eq!(back.resource, Resource::Cpu);
    }

    #[test]
    fn default_registry() {
        let reg = ModelRegistry::with_defaults();
        assert_eq!(reg.all().len(), 3);
    }

    #[test]
    fn route_by_task() {
        let reg = ModelRegistry::with_defaults();

        let orch = reg.route(TaskType::Orchestration).expect("no orchestration model");
        assert_eq!(orch.name, "router-small");

        let embed = reg.route(TaskType::Embedding).expect("no embedding model");
        assert_eq!(embed.name, "embedder-small");

        let code = reg.route(TaskType::CodeGen).expect("no codegen model");
        assert_eq!(code.name, "coder-medium");
    }

    #[test]
    fn route_with_resource_preference() {
        let reg = ModelRegistry::with_defaults();
        let model = reg
            .route_with_resource(TaskType::Orchestration, Resource::Cpu)
            .expect("no model");
        assert_eq!(model.resource, Resource::Cpu);
    }

    #[test]
    fn by_resource_filter() {
        let reg = ModelRegistry::with_defaults();
        let cpu = reg.by_resource(Resource::Cpu);
        assert_eq!(cpu.len(), 3);
        assert!(cpu.iter().all(|m| m.resource == Resource::Cpu));
    }

    #[test]
    fn get_by_name() {
        let reg = ModelRegistry::with_defaults();
        let model = reg.get("coder-medium").expect("not found");
        assert_eq!(model.max_tokens, 8192);

        assert!(reg.get("nonexistent").is_err());
    }

    #[test]
    fn total_memory_estimate() {
        let reg = ModelRegistry::with_defaults();
        let total = reg.total_memory_mb();
        assert!(total > 0);
    }
}
