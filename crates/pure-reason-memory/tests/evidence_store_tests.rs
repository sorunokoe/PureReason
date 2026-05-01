use pure_reason_memory::{EvidenceRecord, EvidenceStore};
use pure_reason_trace::{TaskId, TraceId};
use pure_reason_verifier::{
    ArtifactKind, Category, Finding, Severity, Verdict, VerificationResult,
};
use tempfile::TempDir;

fn sample_result() -> VerificationResult {
    VerificationResult {
        verdict: Verdict {
            passed: false,
            risk_score: 0.8,
            summary: "Review required".to_string(),
        },
        findings: vec![Finding {
            severity: Severity::Error,
            category: Category::Contradiction,
            message: "Conflicting statements detected".to_string(),
        }],
        regulated_text: Some("A more careful rewrite.".to_string()),
        metadata: Default::default(), // TRIZ metadata
    }
}

#[test]
fn appends_and_reloads_review_records() {
    let temp_dir = TempDir::new().unwrap();
    let store = EvidenceStore::open_local(temp_dir.path()).unwrap();
    let task_id = TaskId::new();
    let trace_id = TraceId::new();
    let record = EvidenceRecord::from_review(
        task_id,
        trace_id,
        ArtifactKind::Text,
        "awaiting_review",
        "The sky is green.",
        Some(&sample_result()),
        None,
    );

    store.append(&record).unwrap();

    let recent = store.list_recent(10).unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].trace_id, trace_id);
    assert_eq!(recent[0].summary.as_deref(), Some("Review required"));
    assert_eq!(recent[0].content_hash, record.content_hash);
    assert_eq!(
        recent[0].regulated_text.as_deref(),
        Some("A more careful rewrite.")
    );
}

#[test]
fn list_by_trace_filters_records() {
    let store = EvidenceStore::open_in_memory().unwrap();
    let trace_a = TraceId::new();
    let trace_b = TraceId::new();

    let first = EvidenceRecord::from_review(
        TaskId::new(),
        trace_a,
        ArtifactKind::Text,
        "completed",
        "A",
        Some(&sample_result()),
        None,
    );
    let second = EvidenceRecord::from_review(
        TaskId::new(),
        trace_b,
        ArtifactKind::Text,
        "failed",
        "B",
        None,
        Some("bad input".to_string()),
    );

    store.append(&first).unwrap();
    store.append(&second).unwrap();

    let filtered = store.list_by_trace(&trace_a, 10).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].trace_id, trace_a);
    assert_eq!(filtered[0].error, None);
}
