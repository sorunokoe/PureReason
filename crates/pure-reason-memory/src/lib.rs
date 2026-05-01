//! Local immutable review evidence store for PureReason.

pub mod error;
pub mod store;

use chrono::Utc;
use pure_reason_trace::{EvidenceId, TaskId, TraceId};
use pure_reason_verifier::{ArtifactKind, Finding, VerificationResult};
use serde::{Deserialize, Serialize};

pub use error::{MemoryError, Result};
pub use store::{EvidenceStore, DEFAULT_LIMIT, EVIDENCE_DB_FILE_NAME, MAX_LIMIT};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub evidence_id: EvidenceId,
    pub task_id: TaskId,
    pub trace_id: TraceId,
    pub artifact_kind: ArtifactKind,
    pub final_state: String,
    pub content: String,
    pub content_hash: String,
    pub passed: Option<bool>,
    pub risk_score: Option<f64>,
    pub summary: Option<String>,
    pub findings: Vec<Finding>,
    pub regulated_text: Option<String>,
    pub error: Option<String>,
    pub created_at: String,
}

impl EvidenceRecord {
    pub fn from_review(
        task_id: TaskId,
        trace_id: TraceId,
        artifact_kind: ArtifactKind,
        final_state: impl Into<String>,
        content: impl Into<String>,
        verification: Option<&VerificationResult>,
        error: Option<String>,
    ) -> Self {
        let content = content.into();
        let (passed, risk_score, summary, findings, regulated_text) = verification
            .map(|result| {
                (
                    Some(result.verdict.passed),
                    Some(result.verdict.risk_score),
                    Some(result.verdict.summary.clone()),
                    result.findings.clone(),
                    result.regulated_text.clone(),
                )
            })
            .unwrap_or_else(|| (None, None, None, Vec::new(), None));

        Self {
            evidence_id: EvidenceId::new(),
            task_id,
            trace_id,
            artifact_kind,
            final_state: final_state.into(),
            content_hash: blake3::hash(content.as_bytes()).to_hex().to_string(),
            content,
            passed,
            risk_score,
            summary,
            findings,
            regulated_text,
            error,
            created_at: Utc::now().to_rfc3339(),
        }
    }
}
