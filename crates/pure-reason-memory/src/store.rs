use crate::{error::Result, EvidenceRecord, MemoryError};
use pure_reason_trace::{EvidenceId, TaskId, TraceId};
use pure_reason_verifier::{ArtifactKind, Finding};
use rusqlite::{params, Connection};
use std::{
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
};

pub const EVIDENCE_DB_FILE_NAME: &str = "evidence.sqlite3";
pub const DEFAULT_LIMIT: usize = 100;
pub const MAX_LIMIT: usize = 1000;

const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS review_evidence (
        evidence_id    TEXT PRIMARY KEY,
        task_id        TEXT NOT NULL,
        trace_id       TEXT NOT NULL,
        artifact_kind  TEXT NOT NULL,
        final_state    TEXT NOT NULL,
        content        TEXT NOT NULL,
        content_hash   TEXT NOT NULL,
        passed         INTEGER,
        risk_score     REAL,
        summary        TEXT,
        findings_json  TEXT NOT NULL,
        regulated_text TEXT,
        error          TEXT,
        created_at     TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_review_evidence_recent
        ON review_evidence(created_at DESC);
    CREATE INDEX IF NOT EXISTS idx_review_evidence_trace
        ON review_evidence(trace_id, created_at ASC);
    CREATE INDEX IF NOT EXISTS idx_review_evidence_task
        ON review_evidence(task_id, created_at ASC);
";

fn clamp(limit: usize) -> usize {
    limit.clamp(1, MAX_LIMIT)
}

fn deserialize_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<EvidenceRecord> {
    let evidence_id_str: String = row.get(0)?;
    let task_id_str: String = row.get(1)?;
    let trace_id_str: String = row.get(2)?;
    let artifact_kind_json: String = row.get(3)?;
    let final_state: String = row.get(4)?;
    let content: String = row.get(5)?;
    let content_hash: String = row.get(6)?;
    let passed: Option<i64> = row.get(7)?;
    let risk_score: Option<f64> = row.get(8)?;
    let summary: Option<String> = row.get(9)?;
    let findings_json: String = row.get(10)?;
    let regulated_text: Option<String> = row.get(11)?;
    let error: Option<String> = row.get(12)?;
    let created_at: String = row.get(13)?;

    Ok(EvidenceRecord {
        evidence_id: evidence_id_str
            .parse::<EvidenceId>()
            .map_err(from_parse_error)?,
        task_id: task_id_str.parse::<TaskId>().map_err(from_parse_error)?,
        trace_id: trace_id_str.parse::<TraceId>().map_err(from_parse_error)?,
        artifact_kind: serde_json::from_str::<ArtifactKind>(&artifact_kind_json)
            .map_err(from_json_error)?,
        final_state,
        content,
        content_hash,
        passed: passed.map(|value| value != 0),
        risk_score,
        summary,
        findings: serde_json::from_str::<Vec<Finding>>(&findings_json).map_err(from_json_error)?,
        regulated_text,
        error,
        created_at,
    })
}

fn from_parse_error(error: uuid::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

fn from_json_error(error: serde_json::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

#[derive(Clone)]
pub struct EvidenceStore {
    conn: Arc<Mutex<Connection>>,
}

impl EvidenceStore {
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

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn open_local(dir: impl AsRef<Path>) -> Result<Self> {
        Self::open(dir.as_ref().join(EVIDENCE_DB_FILE_NAME))
    }

    fn connection(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| MemoryError::Poisoned("evidence store connection".to_string()))
    }

    pub fn append(&self, record: &EvidenceRecord) -> Result<()> {
        let artifact_kind_json = serde_json::to_string(&record.artifact_kind)?;
        let findings_json = serde_json::to_string(&record.findings)?;
        let conn = self.connection()?;
        conn.execute(
            "INSERT OR IGNORE INTO review_evidence
             (evidence_id, task_id, trace_id, artifact_kind, final_state, content, content_hash,
              passed, risk_score, summary, findings_json, regulated_text, error, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                record.evidence_id.to_string(),
                record.task_id.to_string(),
                record.trace_id.to_string(),
                artifact_kind_json,
                record.final_state.as_str(),
                record.content.as_str(),
                record.content_hash.as_str(),
                record.passed.map(|value| if value { 1_i64 } else { 0_i64 }),
                record.risk_score,
                record.summary.as_deref(),
                findings_json,
                record.regulated_text.as_deref(),
                record.error.as_deref(),
                record.created_at.as_str(),
            ],
        )?;
        Ok(())
    }

    pub fn list_recent(&self, limit: usize) -> Result<Vec<EvidenceRecord>> {
        let limit = clamp(limit) as i64;
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT evidence_id, task_id, trace_id, artifact_kind, final_state, content,
                    content_hash, passed, risk_score, summary, findings_json,
                    regulated_text, error, created_at
             FROM review_evidence
             ORDER BY created_at DESC
             LIMIT ?1",
        )?;
        let records: std::result::Result<Vec<_>, _> = stmt
            .query_map(params![limit], deserialize_record)?
            .collect();
        records.map_err(MemoryError::from)
    }

    pub fn list_by_trace(&self, trace_id: &TraceId, limit: usize) -> Result<Vec<EvidenceRecord>> {
        let limit = clamp(limit) as i64;
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT evidence_id, task_id, trace_id, artifact_kind, final_state, content,
                    content_hash, passed, risk_score, summary, findings_json,
                    regulated_text, error, created_at
             FROM review_evidence
             WHERE trace_id = ?1
             ORDER BY created_at ASC
             LIMIT ?2",
        )?;
        let records: std::result::Result<Vec<_>, _> = stmt
            .query_map(params![trace_id.to_string(), limit], deserialize_record)?
            .collect();
        records.map_err(MemoryError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[test]
    fn poisoned_mutex_returns_memory_error() {
        let store = EvidenceStore::open_in_memory().unwrap();
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = store.conn.lock().unwrap();
            panic!("poison evidence store");
        }));

        let error = store.list_recent(1).unwrap_err();
        assert!(matches!(error, MemoryError::Poisoned(_)));
    }
}
