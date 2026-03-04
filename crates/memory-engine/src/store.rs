//! Stockage mémoire SQLite WAL avec support TOON.

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
    #[error("toon error: {0}")]
    Toon(#[from] toon_core::ToonError),
}

pub type MemoryResult<T> = Result<T, MemoryError>;

/// Fragment mémoire persisté.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
                toon_data  BLOB,
                is_toon    INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    /// Insère un fragment, compresse en TOON si > 100 tokens.
    pub fn insert(&self, fragment: &MemoryFragment) -> MemoryResult<()> {
        let (data, is_toon) = toon_core::smart_encode(fragment)?;
        self.conn.execute(
            "INSERT INTO fragments (id, session_id, content, toon_data, is_toon, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                fragment.id,
                fragment.session_id,
                fragment.content,
                data,
                is_toon as i32,
                fragment.created_at
            ],
        )?;
        Ok(())
    }

    /// Récupère tous les fragments d'une session.
    pub fn get_by_session(&self, session_id: &str) -> MemoryResult<Vec<MemoryFragment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, content, toon_data, is_toon, created_at
             FROM fragments WHERE session_id = ?1 ORDER BY created_at",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(MemoryFragment {
                id: row.get(0)?,
                session_id: row.get(1)?,
                content: row.get(2)?,
                created_at: row.get(5)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Récupère un fragment par ID.
    pub fn get_by_id(&self, id: &str) -> MemoryResult<Option<MemoryFragment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, content, created_at FROM fragments WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(MemoryFragment {
                id: row.get(0)?,
                session_id: row.get(1)?,
                content: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    /// Supprime un fragment par ID.
    pub fn delete(&self, id: &str) -> MemoryResult<bool> {
        let affected = self.conn.execute(
            "DELETE FROM fragments WHERE id = ?1",
            params![id],
        )?;
        Ok(affected > 0)
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

    /// Recherche par contenu (LIKE %query%).
    pub fn search(&self, query: &str) -> MemoryResult<Vec<MemoryFragment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, content, created_at
             FROM fragments WHERE content LIKE ?1 ORDER BY created_at DESC",
        )?;
        let pattern = format!("%{query}%");
        let rows = stmt.query_map(params![pattern], |row| {
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

    /// Liste toutes les sessions distinctes.
    pub fn list_sessions(&self) -> MemoryResult<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT session_id FROM fragments ORDER BY session_id",
        )?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fragment(id: &str, session: &str, content: &str) -> MemoryFragment {
        MemoryFragment {
            id: id.to_string(),
            session_id: session.to_string(),
            content: content.to_string(),
            created_at: "2026-03-04T00:00:00".to_string(),
        }
    }

    #[test]
    fn insert_and_retrieve() {
        let store = MemoryStore::open(":memory:").expect("open failed");
        let fragment = make_fragment("f1", "s1", "test memory");
        store.insert(&fragment).expect("insert failed");

        let results = store.get_by_session("s1").expect("get failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "test memory");
    }

    #[test]
    fn count_fragments() {
        let store = MemoryStore::open(":memory:").expect("open failed");
        assert_eq!(store.count().expect("count failed"), 0);

        store.insert(&make_fragment("f1", "s1", "data")).expect("insert failed");
        assert_eq!(store.count().expect("count failed"), 1);
    }

    #[test]
    fn get_by_id() {
        let store = MemoryStore::open(":memory:").expect("open failed");
        store.insert(&make_fragment("f1", "s1", "findme")).expect("insert failed");

        let found = store.get_by_id("f1").expect("get_by_id failed");
        assert!(found.is_some());
        assert_eq!(found.as_ref().map(|f| f.content.as_str()), Some("findme"));

        let missing = store.get_by_id("nope").expect("get_by_id failed");
        assert!(missing.is_none());
    }

    #[test]
    fn delete_fragment() {
        let store = MemoryStore::open(":memory:").expect("open failed");
        store.insert(&make_fragment("f1", "s1", "bye")).expect("insert failed");

        assert!(store.delete("f1").expect("delete failed"));
        assert!(!store.delete("f1").expect("delete failed"));
        assert_eq!(store.count().expect("count failed"), 0);
    }

    #[test]
    fn search_content() {
        let store = MemoryStore::open(":memory:").expect("open failed");
        store.insert(&make_fragment("f1", "s1", "hello world")).expect("insert failed");
        store.insert(&make_fragment("f2", "s1", "goodbye moon")).expect("insert failed");

        let results = store.search("hello").expect("search failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "f1");
    }

    #[test]
    fn list_sessions() {
        let store = MemoryStore::open(":memory:").expect("open failed");
        store.insert(&make_fragment("f1", "s1", "a")).expect("insert failed");
        store.insert(&make_fragment("f2", "s2", "b")).expect("insert failed");
        store.insert(&make_fragment("f3", "s1", "c")).expect("insert failed");

        let sessions = store.list_sessions().expect("list failed");
        assert_eq!(sessions, vec!["s1", "s2"]);
    }

    #[test]
    fn large_content_uses_toon() {
        let store = MemoryStore::open(":memory:").expect("open failed");
        let big_content = "word ".repeat(200);
        store.insert(&make_fragment("big", "s1", &big_content)).expect("insert failed");

        let results = store.get_by_session("s1").expect("get failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, big_content);
    }
}
