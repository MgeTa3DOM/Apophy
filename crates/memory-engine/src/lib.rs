//! Memory Engine — SQLite WAL persistent memory.
//!
//! Chaque session laisse une trace. Zéro fragmentation inter-sessions.

pub mod store;

pub use store::MemoryStore;
