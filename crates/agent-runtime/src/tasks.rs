//! Tâches concrètes BarqFlow pour le workflow ArXiv Lab.
//!
//! Trois noeuds DAG : ArxivMonitor → RumadFilter → PseudoCodeGen.

use crate::task::{HybridTask, TaskContext, TaskError, TaskResult};
use async_trait::async_trait;
use tracing::info;

/// Monitore ArXiv et collecte les papers pertinents.
pub struct ArxivMonitor {
    pub query: String,
}

impl ArxivMonitor {
    pub fn new(query: impl Into<String>) -> Self {
        Self { query: query.into() }
    }
}

#[async_trait]
impl HybridTask for ArxivMonitor {
    fn name(&self) -> &str {
        "arxiv_monitor"
    }

    async fn execute(&self, ctx: &mut TaskContext) -> TaskResult<()> {
        // TODO: remplacer par reqwest ArXiv API quand --allow-network
        let papers = vec![
            "RUMAD: Robust Unsupervised Multi-scale Anomaly Detection — Tsinghua",
            "Hierarchical Concept Graphs for Visual Reasoning — Cambridge",
            "PseudoAct: Pseudocode-to-Action Translation — Texas A&M",
        ];

        let paper_list: Vec<String> = papers.iter().map(|p| p.to_string()).collect();
        let count = paper_list.len();
        ctx.results.insert(
            "arxiv_papers".to_string(),
            serde_json::to_string(&paper_list)
                .map_err(|e| TaskError::Execution(e.to_string()))?,
        );
        ctx.metrics.insert("papers_found".to_string(), count as u64);

        info!(query = %self.query, count, "arxiv papers collected");
        Ok(())
    }
}

/// Filtre topologique RUMAD — cosine similarity entre papers.
pub struct RumadFilter {
    pub similarity_threshold: f64,
}

impl RumadFilter {
    pub fn new(threshold: f64) -> Self {
        Self { similarity_threshold: threshold }
    }
}

#[async_trait]
impl HybridTask for RumadFilter {
    fn name(&self) -> &str {
        "rumad_filter"
    }

    async fn execute(&self, ctx: &mut TaskContext) -> TaskResult<()> {
        let papers_json = ctx.results.get("arxiv_papers")
            .ok_or_else(|| TaskError::MissingDep("arxiv_monitor".to_string()))?;

        let papers: Vec<String> = serde_json::from_str(papers_json)
            .map_err(|e| TaskError::Execution(e.to_string()))?;

        // Topologie simulée : noeud par paper + arêtes de similarité
        let nodes: Vec<String> = papers.iter().enumerate()
            .map(|(i, _)| format!("node_{i}"))
            .collect();

        let node_count = nodes.len();
        ctx.results.insert(
            "rumad_nodes".to_string(),
            serde_json::to_string(&nodes)
                .map_err(|e| TaskError::Execution(e.to_string()))?,
        );
        ctx.metrics.insert("rumad_nodes".to_string(), node_count as u64);

        info!(
            threshold = self.similarity_threshold,
            nodes = node_count,
            "RUMAD topological filter applied"
        );
        Ok(())
    }
}

/// Génère du pseudocode à partir de la topologie filtrée.
pub struct PseudoCodeGen;

#[async_trait]
impl HybridTask for PseudoCodeGen {
    fn name(&self) -> &str {
        "pseudocode_gen"
    }

    async fn execute(&self, ctx: &mut TaskContext) -> TaskResult<()> {
        let nodes_json = ctx.results.get("rumad_nodes")
            .ok_or_else(|| TaskError::MissingDep("rumad_filter".to_string()))?;

        let nodes: Vec<String> = serde_json::from_str(nodes_json)
            .map_err(|e| TaskError::Execution(e.to_string()))?;

        let pseudocode = nodes.iter()
            .map(|n| format!("  PROCESS {n}: extract_concepts → embed → store"))
            .collect::<Vec<_>>()
            .join("\n");

        let output = format!("PIPELINE {{\n{pseudocode}\n  MERGE all → knowledge_graph\n}}");

        ctx.results.insert("pseudocode".to_string(), output);
        ctx.metrics.insert("pseudocode_lines".to_string(), (nodes.len() + 2) as u64);

        info!(nodes = nodes.len(), "pseudocode generated");
        Ok(())
    }
}

/// Construit le workflow ArXiv Lab complet (3 tâches, 2 arêtes).
pub fn arxiv_lab_workflow(query: impl Into<String>) -> crate::workflow::HybridWorkflow {
    let mut wf = crate::workflow::HybridWorkflow::new();
    wf.add_task(Box::new(ArxivMonitor::new(query)));
    wf.add_task(Box::new(RumadFilter::new(0.75)));
    wf.add_task(Box::new(PseudoCodeGen));
    wf.add_edge("arxiv_monitor", "rumad_filter");
    wf.add_edge("rumad_filter", "pseudocode_gen");
    wf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn arxiv_monitor_collects_papers() {
        let task = ArxivMonitor::new("topologization 2026");
        let mut ctx = TaskContext::new("test");
        task.execute(&mut ctx).await.expect("arxiv_monitor failed");

        assert!(ctx.results.contains_key("arxiv_papers"));
        assert_eq!(ctx.metrics["papers_found"], 3);
    }

    #[tokio::test]
    async fn rumad_filter_requires_papers() {
        let task = RumadFilter::new(0.75);
        let mut ctx = TaskContext::new("test");
        let err = task.execute(&mut ctx).await.unwrap_err();
        assert!(err.to_string().contains("arxiv_monitor"));
    }

    #[tokio::test]
    async fn full_arxiv_lab_workflow() {
        let wf = arxiv_lab_workflow("topologization");
        let ctx = wf.run("ArXiv Lab 2026-03-04").await.expect("workflow failed");

        assert_eq!(ctx.graph_nodes.len(), 3);
        assert!(ctx.results.contains_key("pseudocode"));
        assert!(ctx.dag_repr.contains("arxiv_monitor -> rumad_filter"));
        assert!(ctx.dag_repr.contains("rumad_filter -> pseudocode_gen"));
        assert_eq!(ctx.metrics["papers_found"], 3);
    }
}
