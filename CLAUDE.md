# APOPHY — AI Continuity Engine

## MISSION

Persistent AI memory and workflow orchestration. Every session leaves a trace.
Every trace improves the next. Memory fragmentation is solved by design.

## CORE PROBLEMS SOLVED

1. **Silent model degradation** — watchdog + automatic fallback
2. **Inter-session memory fragmentation** — TOON + SQLite WAL
3. **Quality regression** — AZR self-play + autorecursive refine loop
4. **Vendor lock-in** — pluggable backends, local-first, any GGUF model

## ABSOLUTE RULES

- `cargo test` must pass before every commit
- Never `unwrap()` in production — always `Result<T, AppError>`
- Each crate = single responsibility
- TOON mandatory for serialization > 100 tokens
- Logs: `tracing::info/warn/error` only, never `println!` in prod

## STACK

- Rust 2021 / Tokio 1.40 / Axum 0.8
- SQLite WAL via rusqlite 0.32 (persistent memory)
- TOON v3 + zstd-12 (compact serialization, -60% tokens)
- Pluggable LLM backend (llama.cpp, vLLM, Ollama, or any GGUF runtime)

## ARCHITECTURE — 7 CRATES

```
toon-core → memory-engine → llm-router → prompt-engine → refine-loop → agent-runtime → api-server
```

Each arrow = direct dependency. Direction = data flow.

## CODE CONVENTIONS

- `snake_case` Rust, `camelCase` TypeScript
- Errors: `Result<T, AppError>` everywhere
- Logs: `tracing::info/warn/error` only
- Tests: a `#[cfg(test)]` module per file minimum
- Commits: `feat:`, `fix:`, `refactor:`, `perf:`, `docs:`

## DEFINITION OF DONE

- [ ] Tests green (`cargo test --workspace`)
- [ ] TOON roundtrip validated for any new structure
- [ ] Memory persist/load tested
- [ ] Zero `unwrap()` in prod (grep audit)
- [ ] Inline doc on all public functions

## DEVELOPMENT PHASES

```
Phase 1 — Foundations    : toon-core, memory-engine
Phase 2 — Inference      : llm-router, agent-runtime
Phase 3 — Intelligence   : prompt-engine (AZR), refine-loop
Phase 4 — Interface      : api-server, dashboard
Phase 5 — Self-Improve   : auto-dataset, auto-finetune, router.reload()
```
