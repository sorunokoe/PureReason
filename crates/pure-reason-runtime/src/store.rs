//! SQLite-backed local task store.
//!
//! Schema
//! ------
//! ```sql
//! CREATE TABLE runtime_tasks (
//!     task_id     TEXT PRIMARY KEY,
//!     trace_id    TEXT NOT NULL,
//!     kind        TEXT NOT NULL,
//!     state       TEXT NOT NULL,
//!     description TEXT NOT NULL,
//!     created_at  TEXT NOT NULL,
//!     updated_at  TEXT NOT NULL
//! );
//! CREATE TABLE runtime_transitions (
//!     id         INTEGER PRIMARY KEY AUTOINCREMENT,
//!     task_id    TEXT NOT NULL,
//!     from_state TEXT NOT NULL,
//!     to_state   TEXT NOT NULL,
//!     timestamp  TEXT NOT NULL,
//!     reason     TEXT
//! );
//! ```

use crate::{
    error::{Result, RuntimeError},
    types::{StateTransition, Task, TaskState, WorkflowKind},
};
use chrono::Utc;
use pure_reason_trace::ids::{TaskId, TraceId};
use rusqlite::{params, Connection};
use std::{
    path::Path,
    str::FromStr,
    sync::{Arc, Mutex, MutexGuard},
};

const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS runtime_tasks (
        task_id     TEXT PRIMARY KEY,
        trace_id    TEXT NOT NULL,
        kind        TEXT NOT NULL,
        state       TEXT NOT NULL,
        description TEXT NOT NULL,
        created_at  TEXT NOT NULL,
        updated_at  TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS runtime_transitions (
        id         INTEGER PRIMARY KEY AUTOINCREMENT,
        task_id    TEXT NOT NULL,
        from_state TEXT NOT NULL,
        to_state   TEXT NOT NULL,
        timestamp  TEXT NOT NULL,
        reason     TEXT
    );
    CREATE INDEX IF NOT EXISTS idx_runtime_tasks_recent
        ON runtime_tasks(created_at DESC);
    CREATE INDEX IF NOT EXISTS idx_runtime_transitions_task
        ON runtime_transitions(task_id, id ASC);
";

pub const DEFAULT_LIMIT: usize = 100;
pub const MAX_LIMIT: usize = 1000;

fn clamp(limit: usize) -> usize {
    limit.clamp(1, MAX_LIMIT)
}

fn deserialize_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    let task_id_str: String = row.get(0)?;
    let trace_id_str: String = row.get(1)?;
    let kind_json: String = row.get(2)?;
    let state_str: String = row.get(3)?;
    let description: String = row.get(4)?;
    let created_at: String = row.get(5)?;
    let updated_at: String = row.get(6)?;

    Ok(Task {
        task_id: task_id_str.parse::<TaskId>().map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(UuidErr(e.to_string())),
            )
        })?,
        trace_id: trace_id_str.parse::<TraceId>().map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                Box::new(UuidErr(e.to_string())),
            )
        })?,
        kind: serde_json::from_str::<WorkflowKind>(&kind_json).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                Box::new(UuidErr(e.to_string())),
            )
        })?,
        state: TaskState::from_str(&state_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                3,
                rusqlite::types::Type::Text,
                Box::new(UuidErr(e)),
            )
        })?,
        description,
        created_at,
        updated_at,
    })
}

/// Small helper to box arbitrary string errors for rusqlite conversion errors.
#[derive(Debug)]
struct UuidErr(String);
impl std::fmt::Display for UuidErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for UuidErr {}

/// SQLite-backed store for runtime tasks and their transition history.
#[derive(Clone)]
pub struct TaskStore {
    conn: Arc<Mutex<Connection>>,
}

impl TaskStore {
    /// Open (or create) a task store at `path`.
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

    /// Open an in-memory store. Useful for tests.
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
            .map_err(|_| RuntimeError::Poisoned("task store connection".to_string()))
    }

    /// Persist a new task. The task must be in the `Created` state.
    pub fn create_task(&self, task: &Task) -> Result<()> {
        let kind_json = serde_json::to_string(&task.kind)?;
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO runtime_tasks
             (task_id, trace_id, kind, state, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                task.task_id.to_string(),
                task.trace_id.to_string(),
                kind_json,
                task.state.to_string(),
                task.description,
                task.created_at,
                task.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Load a task by its ID.
    pub fn load_task(&self, task_id: &TaskId) -> Result<Task> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT task_id, trace_id, kind, state, description, created_at, updated_at
             FROM runtime_tasks WHERE task_id = ?1",
        )?;
        let mut rows = stmt.query_map(params![task_id.to_string()], deserialize_task)?;
        rows.next()
            .ok_or_else(|| RuntimeError::TaskNotFound(task_id.to_string()))?
            .map_err(RuntimeError::from)
    }

    /// Transition `task_id` from its current state to `to_state`.
    ///
    /// Returns an error if the transition is not valid or the task is already in
    /// a terminal state.
    pub fn transition(
        &self,
        task_id: &TaskId,
        to_state: TaskState,
        reason: Option<&str>,
    ) -> Result<StateTransition> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT task_id, trace_id, kind, state, description, created_at, updated_at
             FROM runtime_tasks WHERE task_id = ?1",
        )?;
        let mut rows = stmt.query_map(params![task_id.to_string()], deserialize_task)?;
        let task = rows
            .next()
            .ok_or_else(|| RuntimeError::TaskNotFound(task_id.to_string()))?
            .map_err(RuntimeError::from)?;

        let from_state = task.state;
        if !from_state.can_transition_to(to_state) {
            return Err(RuntimeError::InvalidTransition {
                from: from_state.to_string(),
                to: to_state.to_string(),
            });
        }

        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE runtime_tasks SET state = ?1, updated_at = ?2 WHERE task_id = ?3",
            params![to_state.to_string(), now, task_id.to_string()],
        )?;

        conn.execute(
            "INSERT INTO runtime_transitions (task_id, from_state, to_state, timestamp, reason)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                task_id.to_string(),
                from_state.to_string(),
                to_state.to_string(),
                now,
                reason,
            ],
        )?;

        Ok(StateTransition {
            task_id: *task_id,
            from_state,
            to_state,
            timestamp: now,
            reason: reason.map(str::to_string),
        })
    }

    /// Return recently created tasks, newest first.
    pub fn list_recent(&self, limit: usize) -> Result<Vec<Task>> {
        let limit = clamp(limit) as i64;
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT task_id, trace_id, kind, state, description, created_at, updated_at
             FROM runtime_tasks ORDER BY created_at DESC LIMIT ?1",
        )?;
        let tasks: std::result::Result<Vec<Task>, _> =
            stmt.query_map(params![limit], deserialize_task)?.collect();
        tasks.map_err(RuntimeError::from)
    }

    /// Return the full transition history for `task_id`, oldest first.
    pub fn list_transitions(&self, task_id: &TaskId) -> Result<Vec<StateTransition>> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT task_id, from_state, to_state, timestamp, reason
             FROM runtime_transitions WHERE task_id = ?1 ORDER BY id ASC",
        )?;
        let rows: std::result::Result<Vec<_>, _> = stmt
            .query_map(params![task_id.to_string()], |row| {
                let task_id_str: String = row.get(0)?;
                let from_str: String = row.get(1)?;
                let to_str: String = row.get(2)?;
                let timestamp: String = row.get(3)?;
                let reason: Option<String> = row.get(4)?;
                Ok((task_id_str, from_str, to_str, timestamp, reason))
            })?
            .collect();

        rows.map_err(RuntimeError::from)?
            .into_iter()
            .map(|(tid, from_str, to_str, timestamp, reason)| {
                Ok(StateTransition {
                    task_id: tid
                        .parse::<TaskId>()
                        .map_err(|e| RuntimeError::Sqlite(e.to_string()))?,
                    from_state: TaskState::from_str(&from_str).map_err(RuntimeError::Sqlite)?,
                    to_state: TaskState::from_str(&to_str).map_err(RuntimeError::Sqlite)?,
                    timestamp,
                    reason,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[test]
    fn poisoned_mutex_returns_runtime_error() {
        let store = TaskStore::open_in_memory().unwrap();
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = store.conn.lock().unwrap();
            panic!("poison runtime store");
        }));

        let error = store.list_recent(1).unwrap_err();
        assert!(matches!(error, RuntimeError::Poisoned(_)));
    }
}
