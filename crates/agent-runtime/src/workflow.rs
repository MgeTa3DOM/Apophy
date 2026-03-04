//! Workflow hybride BarqFlow + graph-flow.
//!
//! Orchestre l'exécution séquentielle de tâches selon un DAG.
//! Chaque tâche est un noeud, les arêtes sont les dépendances.

use crate::task::{HybridTask, TaskContext, TaskError, TaskResult};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::toposort;
use std::collections::HashMap;
use tracing::{info, error};

/// Workflow hybride : enregistre des tâches, construit un DAG, exécute en ordre topologique.
pub struct HybridWorkflow {
    tasks: Vec<Box<dyn HybridTask>>,
    /// Arêtes de dépendance : (from_name, to_name).
    edges: Vec<(String, String)>,
}

impl HybridWorkflow {
    /// Crée un workflow vide.
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Ajoute une tâche au workflow.
    pub fn add_task(&mut self, task: Box<dyn HybridTask>) {
        self.tasks.push(task);
    }

    /// Déclare une dépendance : `from` doit s'exécuter avant `to`.
    pub fn add_edge(&mut self, from: impl Into<String>, to: impl Into<String>) {
        self.edges.push((from.into(), to.into()));
    }

    /// Construit le DAG et retourne l'ordre d'exécution topologique.
    fn build_execution_order(&self) -> TaskResult<Vec<usize>> {
        let mut graph = DiGraph::<&str, ()>::new();
        let mut name_to_node: HashMap<&str, NodeIndex> = HashMap::new();
        let mut name_to_idx: HashMap<&str, usize> = HashMap::new();

        // Enregistrer tous les noeuds
        for (idx, task) in self.tasks.iter().enumerate() {
            let node = graph.add_node(task.name());
            name_to_node.insert(task.name(), node);
            name_to_idx.insert(task.name(), idx);
        }

        // Ajouter les arêtes
        for (from, to) in &self.edges {
            let from_node = name_to_node.get(from.as_str())
                .ok_or_else(|| TaskError::MissingDep(from.clone()))?;
            let to_node = name_to_node.get(to.as_str())
                .ok_or_else(|| TaskError::MissingDep(to.clone()))?;
            graph.add_edge(*from_node, *to_node, ());
        }

        // Tri topologique
        let sorted = toposort(&graph, None)
            .map_err(|_| TaskError::Execution("cyclic dependency detected in DAG".to_string()))?;

        let order: Vec<usize> = sorted.iter()
            .filter_map(|node| {
                let name = graph[*node];
                name_to_idx.get(name).copied()
            })
            .collect();

        Ok(order)
    }

    /// Exécute le workflow complet : tri topologique → exécution séquentielle.
    pub async fn run(&self, input: impl Into<String>) -> TaskResult<TaskContext> {
        let mut ctx = TaskContext::new(input);
        let order = self.build_execution_order()?;

        // Génère la représentation DAG textuelle
        let dag_lines: Vec<String> = self.edges.iter()
            .map(|(from, to)| format!("  {from} -> {to};"))
            .collect();
        ctx.dag_repr = format!("DAG {{\n{}\n}}", dag_lines.join("\n"));

        info!("workflow started — {} tasks, {} edges", self.tasks.len(), self.edges.len());

        for idx in order {
            let task = &self.tasks[idx];
            info!(task = task.name(), "executing");

            match task.execute(&mut ctx).await {
                Ok(()) => {
                    ctx.graph_nodes.push(task.name().to_string());
                    info!(task = task.name(), "completed");
                }
                Err(e) => {
                    error!(task = task.name(), error = %e, "failed");
                    return Err(e);
                }
            }
        }

        info!("workflow completed — {} nodes executed", ctx.graph_nodes.len());
        Ok(ctx)
    }
}

impl Default for HybridWorkflow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::HybridTask;
    use async_trait::async_trait;

    struct StepA;
    #[async_trait]
    impl HybridTask for StepA {
        fn name(&self) -> &str { "step_a" }
        async fn execute(&self, ctx: &mut TaskContext) -> TaskResult<()> {
            ctx.results.insert("a".to_string(), "done".to_string());
            Ok(())
        }
    }

    struct StepB;
    #[async_trait]
    impl HybridTask for StepB {
        fn name(&self) -> &str { "step_b" }
        async fn execute(&self, ctx: &mut TaskContext) -> TaskResult<()> {
            // Vérifie que A a été exécuté avant
            if !ctx.results.contains_key("a") {
                return Err(TaskError::MissingDep("step_a".to_string()));
            }
            ctx.results.insert("b".to_string(), "done".to_string());
            ctx.metrics.insert("tokens_processed".to_string(), 42);
            Ok(())
        }
    }

    struct StepC;
    #[async_trait]
    impl HybridTask for StepC {
        fn name(&self) -> &str { "step_c" }
        async fn execute(&self, ctx: &mut TaskContext) -> TaskResult<()> {
            ctx.results.insert("c".to_string(), "done".to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn sequential_workflow() {
        let mut wf = HybridWorkflow::new();
        wf.add_task(Box::new(StepA));
        wf.add_task(Box::new(StepB));
        wf.add_edge("step_a", "step_b");

        let ctx = wf.run("test").await.expect("workflow failed");

        assert_eq!(ctx.results["a"], "done");
        assert_eq!(ctx.results["b"], "done");
        assert_eq!(ctx.metrics["tokens_processed"], 42);
        assert_eq!(ctx.graph_nodes.len(), 2);
    }

    #[tokio::test]
    async fn dag_with_three_nodes() {
        let mut wf = HybridWorkflow::new();
        wf.add_task(Box::new(StepA));
        wf.add_task(Box::new(StepB));
        wf.add_task(Box::new(StepC));
        wf.add_edge("step_a", "step_b");
        wf.add_edge("step_a", "step_c");

        let ctx = wf.run("multi-path").await.expect("workflow failed");

        assert_eq!(ctx.graph_nodes.len(), 3);
        assert!(ctx.dag_repr.contains("step_a -> step_b"));
        assert!(ctx.dag_repr.contains("step_a -> step_c"));
    }

    #[tokio::test]
    async fn empty_workflow() {
        let wf = HybridWorkflow::new();
        let ctx = wf.run("empty").await.expect("empty workflow failed");
        assert!(ctx.graph_nodes.is_empty());
    }
}
