//! Integration tests for the trace store and serialization contracts.

use pure_reason_trace::{
    ids::{TaskId, TraceId},
    store::TraceStore,
    types::{TraceEvent, TraceEventKind, TraceEventPayload},
};

fn model_call_event(trace_id: TraceId) -> TraceEvent {
    TraceEvent::new(
        trace_id,
        None,
        TraceEventPayload::ModelCall {
            model: "gpt-4o".into(),
            provider: "openai".into(),
            prompt_tokens: 200,
            completion_tokens: 50,
            latency_ms: 320,
            status: "success".into(),
        },
    )
}

fn task_state_event(trace_id: TraceId, task_id: TaskId) -> TraceEvent {
    TraceEvent::new(
        trace_id,
        Some(task_id),
        TraceEventPayload::TaskStateChange {
            task_id,
            from_state: "pending".into(),
            to_state: "running".into(),
        },
    )
}

// ── Store: append + list_by_trace ────────────────────────────────────────

#[test]
fn append_and_list_by_trace() {
    let store = TraceStore::open_in_memory().unwrap();
    let trace_id = TraceId::new();

    let e1 = model_call_event(trace_id);
    let task_id = TaskId::new();
    let e2 = task_state_event(trace_id, task_id);

    store.append(&e1).unwrap();
    store.append(&e2).unwrap();

    let events = store.list_by_trace(&trace_id, 100).unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].trace_id, trace_id);
    assert_eq!(events[1].trace_id, trace_id);
}

#[test]
fn list_by_trace_excludes_other_traces() {
    let store = TraceStore::open_in_memory().unwrap();
    let trace_a = TraceId::new();
    let trace_b = TraceId::new();

    store.append(&model_call_event(trace_a)).unwrap();
    store.append(&model_call_event(trace_b)).unwrap();
    store.append(&model_call_event(trace_b)).unwrap();

    let a_events = store.list_by_trace(&trace_a, 100).unwrap();
    let b_events = store.list_by_trace(&trace_b, 100).unwrap();
    assert_eq!(a_events.len(), 1);
    assert_eq!(b_events.len(), 2);
}

// ── Store: list_recent ────────────────────────────────────────────────────

#[test]
fn list_recent_respects_limit() {
    let store = TraceStore::open_in_memory().unwrap();
    let trace_id = TraceId::new();

    for _ in 0..5 {
        store.append(&model_call_event(trace_id)).unwrap();
    }

    let recent = store.list_recent(3).unwrap();
    assert_eq!(recent.len(), 3);
}

#[test]
fn list_recent_returns_all_when_under_limit() {
    let store = TraceStore::open_in_memory().unwrap();
    let trace_id = TraceId::new();

    store.append(&model_call_event(trace_id)).unwrap();
    store.append(&model_call_event(trace_id)).unwrap();

    let recent = store.list_recent(50).unwrap();
    assert_eq!(recent.len(), 2);
}

// ── Store: idempotent append ──────────────────────────────────────────────

#[test]
fn duplicate_event_id_is_ignored() {
    let store = TraceStore::open_in_memory().unwrap();
    let event = model_call_event(TraceId::new());

    store.append(&event).unwrap();
    store.append(&event).unwrap(); // duplicate — should not error or double-insert

    let recent = store.list_recent(10).unwrap();
    assert_eq!(recent.len(), 1);
}

// ── Store: file-backed persistence ───────────────────────────────────────

#[test]
fn file_backed_store_persists_across_open() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("trace.db");

    let trace_id = TraceId::new();
    {
        let store = TraceStore::open(&db_path).unwrap();
        store.append(&model_call_event(trace_id)).unwrap();
    }
    {
        let store = TraceStore::open(&db_path).unwrap();
        let events = store.list_by_trace(&trace_id, 10).unwrap();
        assert_eq!(events.len(), 1);
    }
}

// ── Serialization round-trips ─────────────────────────────────────────────

#[test]
fn all_event_kinds_round_trip() {
    let trace_id = TraceId::new();
    let task_id = TaskId::new();

    let payloads = vec![
        TraceEventPayload::ModelCall {
            model: "claude-3".into(),
            provider: "anthropic".into(),
            prompt_tokens: 100,
            completion_tokens: 20,
            latency_ms: 150,
            status: "success".into(),
        },
        TraceEventPayload::TaskStateChange {
            task_id,
            from_state: "queued".into(),
            to_state: "done".into(),
        },
        TraceEventPayload::VerificationResult {
            passed: true,
            score: 0.95,
            details: "All checks passed".into(),
        },
        TraceEventPayload::Decision {
            action: "allow".into(),
            rationale: "Score above threshold".into(),
        },
        TraceEventPayload::Note {
            message: "Retrying after timeout".into(),
        },
    ];

    for payload in payloads {
        let event = TraceEvent::new(trace_id, Some(task_id), payload);
        let json = serde_json::to_string(&event).unwrap();
        let decoded: TraceEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.event_id, event.event_id);
        assert_eq!(decoded.trace_id, trace_id);
        assert_eq!(decoded.kind, event.kind);
    }
}

#[test]
fn event_kind_matches_payload() {
    let trace_id = TraceId::new();

    let mc = TraceEvent::new(
        trace_id,
        None,
        TraceEventPayload::ModelCall {
            model: "m".into(),
            provider: "test".into(),
            prompt_tokens: 1,
            completion_tokens: 1,
            latency_ms: 1,
            status: "success".into(),
        },
    );
    assert_eq!(mc.kind, TraceEventKind::ModelCall);

    let note = TraceEvent::new(
        trace_id,
        None,
        TraceEventPayload::Note {
            message: "hi".into(),
        },
    );
    assert_eq!(note.kind, TraceEventKind::Note);
}

// ── ID types ──────────────────────────────────────────────────────────────

#[test]
fn id_display_and_parse_roundtrip() {
    let id = TraceId::new();
    let s = id.to_string();
    let parsed: TraceId = s.parse().unwrap();
    assert_eq!(id, parsed);
}
