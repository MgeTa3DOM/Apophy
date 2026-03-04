# Apophy — Architecture

## System Design

```
┌─────────────────────────────────────────────────────────────────┐
│                        api-server (Axum)                        │
│  REST /api/v1/*  ·  Health /health                              │
├─────────────────────────────────────────────────────────────────┤
│                      agent-runtime (BarqFlow)                   │
│  HybridWorkflow  ·  DAG Executor  ·  Topological Sort           │
├──────────────┬──────────────────────┬───────────────────────────┤
│ prompt-engine│    refine-loop       │     llm-router            │
│ AZR self-play│ Autorecursive refine │ Pluggable model routing   │
├──────────────┴──────────────────────┴───────────────────────────┤
│                      memory-engine (SQLite WAL)                 │
│  Fragments  ·  Sessions  ·  Search  ·  TOON auto-compress       │
├─────────────────────────────────────────────────────────────────┤
│                        toon-core (TOON v3)                      │
│  Encode/Decode  ·  zstd-12  ·  Smart threshold  ·  Batch ops    │
└─────────────────────────────────────────────────────────────────┘
         ↕                                    ↕
   dashboard (Bun + Hono)            finetune (UV + Unsloth)
   Real-time monitoring UI           QLoRA from conversation memory
```

## Stack

| Layer | Technology |
|-------|-----------|
| Core | Rust 2021 / Tokio 1.40 / Axum 0.8 |
| Dashboard | Bun + Hono + TypeScript |
| Fine-Tuning | UV + Unsloth + QLoRA |
| Database | SQLite WAL (rusqlite 0.32) |
| Serialization | TOON v3 + zstd-12 |
| Workflow | petgraph DAG |

## 7 Crates

| Crate | Responsibility |
|-------|---------------|
| `toon-core` | Token-efficient serialization |
| `memory-engine` | Persistent memory (SQLite WAL) |
| `llm-router` | Pluggable model selection |
| `prompt-engine` | AZR self-play refinement |
| `refine-loop` | Autorecursive convergence |
| `agent-runtime` | BarqFlow DAG workflows |
| `api-server` | HTTP API layer |

## Data Flow

```
toon-core → memory-engine → llm-router → prompt-engine → refine-loop → agent-runtime → api-server
```

## Pluggable Backend

```rust
#[async_trait]
pub trait InferenceBackend: Send + Sync {
    async fn generate(&self, model: &str, prompt: &str, max_tokens: usize) -> RouterResult<String>;
    async fn is_healthy(&self, model: &str) -> bool;
}
```

Implement for llama.cpp, vLLM, Ollama, or any runtime.

## Self-Improvement Loop

```
Conversations → memory-engine → finetune/export → QLoRA training → GGUF model → llm-router
```

The model running in 6 months is yours — trained on your own conversations.
