//! SQLite-backed append-only trace store.
//!
//! Schema
//! ------
//! ```sql
//! CREATE TABLE trace_events (
//!     event_id  TEXT PRIMARY KEY,
//!     trace_id  TEXT NOT NULL,
//!     task_id   TEXT,
//!     kind      TEXT NOT NULL,
//!     timestamp TEXT NOT NULL,
//!     data      TEXT NOT NULL   -- full JSON blob
//! );
//! CREATE INDEX idx_trace_events_by_trace ON trace_events(trace_id, timestamp ASC);
//! CREATE INDEX idx_trace_events_recent   ON trace_events(timestamp DESC);
//! ```

use crate::{
    error::{Result, TraceError},
    ids::TraceId,
    types::TraceEvent,
};
use rusqlite::{params, Connection};
use std::{
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
};

pub const DEFAULT_LIMIT: usize = 100;
pub const MAX_LIMIT: usize = 1000;

fn clamp(limit: usize) -> usize {
    limit.clamp(1, MAX_LIMIT)
}

const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS trace_events (
        event_id  TEXT PRIMARY KEY,
        trace_id  TEXT NOT NULL,
        task_id   TEXT,
        kind      TEXT NOT NULL,
        timestamp TEXT NOT NULL,
        data      TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_trace_events_by_trace
        ON trace_events(trace_id, timestamp ASC);
    CREATE INDEX IF NOT EXISTS idx_trace_events_recent
        ON trace_events(timestamp DESC);
";

/// Append-only local trace store backed by SQLite.
///
/// Holds a single persistent connection wrapped in a `Mutex`, which means it
/// is `Send + Sync` and works correctly for both file-backed and in-memory
/// databases.
#[derive(Clone)]
pub struct TraceStore {
    conn: Arc<Mutex<Connection>>,
}

impl TraceStore {
    /// Open (or create) a trace store at `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Open an in-memory store.  Useful for tests and ephemeral contexts.
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn connection(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| TraceError::Poisoned("trace store connection".to_string()))
    }

    /// Append a single event.  Duplicate `event_id`s are silently ignored
    /// (idempotent, safe for at-least-once delivery).
    pub fn append(&self, event: &TraceEvent) -> Result<()> {
        let blob = serde_json::to_string(event)?;
        let conn = self.connection()?;
        conn.execute(
            "INSERT OR IGNORE INTO trace_events
             (event_id, trace_id, task_id, kind, timestamp, data)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                event.event_id.to_string(),
                event.trace_id.to_string(),
                event.task_id.map(|t| t.to_string()),
                event.kind.to_string(),
                event.timestamp,
                blob,
            ],
        )?;
        Ok(())
    }

    /// Return all events for `trace_id`, ordered oldest-first.
    pub fn list_by_trace(&self, trace_id: &TraceId, limit: usize) -> Result<Vec<TraceEvent>> {
        let limit = clamp(limit) as i64;
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM trace_events
             WHERE trace_id = ?1
             ORDER BY timestamp ASC
             LIMIT ?2",
        )?;
        let blobs: Vec<String> = stmt
            .query_map(params![trace_id.to_string(), limit], |row| row.get(0))?
            .collect::<std::result::Result<_, _>>()?;
        blobs
            .iter()
            .map(|b| serde_json::from_str::<TraceEvent>(b).map_err(Into::into))
            .collect()
    }

    /// Return the most recent events across all traces, newest-first.
    pub fn list_recent(&self, limit: usize) -> Result<Vec<TraceEvent>> {
        let limit = clamp(limit) as i64;
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM trace_events
             ORDER BY timestamp DESC
             LIMIT ?1",
        )?;
        let blobs: Vec<String> = stmt
            .query_map(params![limit], |row| row.get(0))?
            .collect::<std::result::Result<_, _>>()?;
        blobs
            .iter()
            .map(|b| serde_json::from_str::<TraceEvent>(b).map_err(Into::into))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[test]
    fn poisoned_mutex_returns_trace_error() {
        let store = TraceStore::open_in_memory().unwrap();
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = store.conn.lock().unwrap();
            panic!("poison trace store");
        }));

        let error = store.list_recent(1).unwrap_err();
        assert!(matches!(error, TraceError::Poisoned(_)));
    }
}
