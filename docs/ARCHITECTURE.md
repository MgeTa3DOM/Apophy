# Apophy — Architecture

## System Design

Apophy is a sovereign local AI continuity engine. Zero cloud. Zero fragmentation. Zero silent degradation.

```
┌─────────────────────────────────────────────────────────────────┐
│                        api-server (Axum)                        │
│  REST /api/v1/*  ·  WebSocket /ws  ·  Health /health            │
├─────────────────────────────────────────────────────────────────┤
│                      agent-runtime (BarqFlow)                   │
│  HybridWorkflow  ·  DAG Executor  ·  Topological Sort           │
├──────────────┬──────────────────────┬───────────────────────────┤
│ prompt-engine│    refine-loop       │     llm-router            │
│ AZR self-play│ Autorecursive refine │ Model registry + thermal  │
├──────────────┴──────────────────────┴───────────────────────────┤
│                      memory-engine (SQLite WAL)                 │
│  Fragments  ·  Sessions  ·  Search  ·  TOON auto-compress       │
├─────────────────────────────────────────────────────────────────┤
│                        toon-core (TOON v3)                      │
│  Encode/Decode  ·  zstd-12  ·  Smart threshold  ·  Batch ops    │
└─────────────────────────────────────────────────────────────────┘
```

## 7 Crates — Single Responsibility

| Crate | Responsibility | Key Type |
|-------|---------------|----------|
| `toon-core` | Token-efficient serialization | `encode()`, `smart_encode()` |
| `memory-engine` | Persistent memory (SQLite WAL) | `MemoryStore`, `MemoryFragment` |
| `llm-router` | Model selection + thermal guard | `ModelRegistry`, `thermal::read_gpu_temp()` |
| `prompt-engine` | AZR self-play prompt refinement | `ScoredPrompt`, `evaluate_prompt()` |
| `refine-loop` | Autorecursive convergence | `refine()`, `RefineConfig` |
| `agent-runtime` | BarqFlow DAG workflow engine | `HybridWorkflow`, `HybridTask` |
| `api-server` | HTTP/WS API layer | `create_router()` |

## Data Flow

```
toon-core → memory-engine → llm-router → prompt-engine → refine-loop → agent-runtime → api-server
```

Each arrow = direct dependency. Direction = data flow.

## Models (Rolling Release Local GGUF)

| Model | Role | Resource | Usage |
|-------|------|----------|-------|
| FunctionGemma:270m | Orchestration (router) | CPU | 100% input |
| GLM-4.7-Flash | Vision / Code | GPU 4GB | 20% max |
| Gemma3:270m | Embedding / Memory | CPU | always |

## Thermal Guard

CLAUDE.md rule: GPU < 80°C or automatic CPU failover.

The `llm-router::thermal` module reads sysfs temperature sensors. When the GPU exceeds the thermal limit, the `ModelRegistry::route()` function automatically excludes GPU models and falls back to CPU-only inference.

## TOON v3 Protocol

Token-Oriented Object Notation — compact serialization for LLMs.

- JSON compact + zstd level 12 compression
- ~60% token reduction vs raw JSON
- Automatic threshold: if content > 100 tokens, TOON is mandatory
- `smart_encode()` handles the decision transparently

## BarqFlow Workflow Engine

Hybrid DAG + graph-flow pattern:

1. Register tasks as DAG nodes
2. Declare edges (dependencies)
3. Topological sort for execution order
4. Sequential execution with shared `TaskContext`
5. Metrics collection at each node
