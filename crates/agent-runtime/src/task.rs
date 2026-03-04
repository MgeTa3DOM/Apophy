//! Task trait et contexte partagé pour le workflow hybride.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Erreurs de tâche.
#[derive(Debug, Error)]
pub enum TaskError {
    #[error("task failed: {0}")]
    Execution(String),
    #[error("missing dependency: {0}")]
    MissingDep(String),
}

pub type TaskResult<T> = Result<T, TaskError>;

/// Contexte partagé entre toutes les tâches du workflow.
///
/// Chaque tâche lit et écrit dans ce contexte.
/// Le DAG garantit l'ordre d'exécution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    /// Input initial du workflow.
    pub input: String,
    /// Résultats intermédiaires (clé = nom tâche, valeur = output).
    pub results: HashMap<String, String>,
    /// Noeuds du graphe topologique.
    pub graph_nodes: Vec<String>,
    /// DAG textuel généré (BarqFlow style).
    pub dag_repr: String,
    /// Métriques collectées.
    pub metrics: HashMap<String, u64>,
}

impl TaskContext {
    /// Crée un nouveau contexte avec l'input initial.
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            results: HashMap::new(),
            graph_nodes: Vec::new(),
            dag_repr: String::new(),
            metrics: HashMap::new(),
        }
    }
}

/// Trait pour une tâche exécutable dans le workflow hybride.
///
/// Implémente le pattern BarqFlow : chaque tâche est un noeud du DAG.
#[async_trait]
pub trait HybridTask: Send + Sync {
    /// Nom unique de la tâche (noeud dans le DAG).
    fn name(&self) -> &str;

    /// Exécute la tâche en mutant le contexte partagé.
    async fn execute(&self, ctx: &mut TaskContext) -> TaskResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_new() {
        let ctx = TaskContext::new("test input");
        assert_eq!(ctx.input, "test input");
        assert!(ctx.results.is_empty());
        assert!(ctx.graph_nodes.is_empty());
    }

    #[test]
    fn context_serialization() {
        let mut ctx = TaskContext::new("serialize me");
        ctx.results.insert("step1".to_string(), "done".to_string());
        ctx.metrics.insert("tokens".to_string(), 42);

        let json = serde_json::to_string(&ctx).expect("serialize failed");
        let back: TaskContext = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(back.results["step1"], "done");
        assert_eq!(back.metrics["tokens"], 42);
    }
}
