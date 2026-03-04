//! Stockage mémoire SQLite WAL.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Erreurs mémoire.
#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type MemoryResult<T> = Result<T, MemoryError>;

/// Fragment mémoire persisté.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFragment {
    pub id: String,
    pub session_id: String,
    pub content: String,
    pub created_at: String,
}

/// Store SQLite WAL pour fragments mémoire.
pub struct MemoryStore {
    conn: Connection,
}

impl MemoryStore {
    /// Ouvre ou crée la base mémoire en mode WAL.
    pub fn open(path: &str) -> MemoryResult<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS fragments (
                id         TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                content    TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    /// Insère un fragment mémoire.
    pub fn insert(&self, fragment: &MemoryFragment) -> MemoryResult<()> {
        self.conn.execute(
            "INSERT INTO fragments (id, session_id, content, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![fragment.id, fragment.session_id, fragment.content, fragment.created_at],
        )?;
        Ok(())
    }

    /// Récupère tous les fragments d'une session.
    pub fn get_by_session(&self, session_id: &str) -> MemoryResult<Vec<MemoryFragment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, content, created_at FROM fragments WHERE session_id = ?1 ORDER BY created_at",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(MemoryFragment {
                id: row.get(0)?,
                session_id: row.get(1)?,
                content: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Compte total de fragments.
    pub fn count(&self) -> MemoryResult<usize> {
        let count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM fragments",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_retrieve() {
        let store = MemoryStore::open(":memory:").expect("open failed");
        let fragment = MemoryFragment {
            id: "f1".to_string(),
            session_id: "s1".to_string(),
            content: "test memory".to_string(),
            created_at: "2026-03-04T00:00:00".to_string(),
        };
        store.insert(&fragment).expect("insert failed");

        let results = store.get_by_session("s1").expect("get failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "test memory");
    }

    #[test]
    fn count_fragments() {
        let store = MemoryStore::open(":memory:").expect("open failed");
        assert_eq!(store.count().expect("count failed"), 0);

        let f = MemoryFragment {
            id: "f1".to_string(),
            session_id: "s1".to_string(),
            content: "data".to_string(),
            created_at: "2026-03-04T00:00:00".to_string(),
        };
        store.insert(&f).expect("insert failed");
        assert_eq!(store.count().expect("count failed"), 1);
    }
}
