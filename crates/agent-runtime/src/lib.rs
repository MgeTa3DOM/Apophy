//! Agent Runtime — Hybrid workflow orchestrator.
//!
//! Combine BarqFlow DAG sequencing + graph-flow topological execution.
//! Pattern: define tasks → build DAG → execute in order → collect results.

pub mod task;
pub mod tasks;
pub mod workflow;

pub use task::{TaskContext, TaskError, TaskResult};
pub use tasks::arxiv_lab_workflow;
pub use workflow::HybridWorkflow;
