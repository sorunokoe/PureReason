//! Integration tests for the runtime crate.

use pure_reason_runtime::{
    executor::{StepOutcome, WorkflowExecutor},
    store::TaskStore,
    types::{TaskState, WorkflowKind},
    RuntimeError,
};
use pure_reason_trace::{ids::TraceId, TraceStore};

// ── Helpers ───────────────────────────────────────────────────────────────

fn stores() -> (TaskStore, TraceStore) {
    (
        TaskStore::open_in_memory().unwrap(),
        TraceStore::open_in_memory().unwrap(),
    )
}

fn executor() -> WorkflowExecutor {
    let (tasks, traces) = stores();
    WorkflowExecutor::new(tasks, traces)
}

// ── TaskStore: persistence ────────────────────────────────────────────────

#[test]
fn store_create_and_load_roundtrip() {
    let (store, _) = stores();
    let trace_id = TraceId::new();

    let task =
        pure_reason_runtime::types::Task::new(trace_id, WorkflowKind::Reasoning, "test task");
    let task_id = task.task_id;
    store.create_task(&task).unwrap();

    let loaded = store.load_task(&task_id).unwrap();
    assert_eq!(loaded.task_id, task_id);
    assert_eq!(loaded.trace_id, trace_id);
    assert_eq!(loaded.state, TaskState::Created);
    assert_eq!(loaded.description, "test task");
}

#[test]
fn store_load_missing_task_returns_error() {
    let (store, _) = stores();
    let task_id = pure_reason_trace::ids::TaskId::new();
    let err = store.load_task(&task_id).unwrap_err();
    assert!(matches!(err, RuntimeError::TaskNotFound(_)));
}

#[test]
fn store_file_backed_persistence() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("runtime.db");
    let trace_id = TraceId::new();

    let task_id = {
        let store = TaskStore::open(&db_path).unwrap();
        let task = pure_reason_runtime::types::Task::new(
            trace_id,
            WorkflowKind::CodeGen,
            "persistent task",
        );
        let id = task.task_id;
        store.create_task(&task).unwrap();
        id
    };

    let store = TaskStore::open(&db_path).unwrap();
    let loaded = store.load_task(&task_id).unwrap();
    assert_eq!(loaded.task_id, task_id);
    assert_eq!(loaded.state, TaskState::Created);
}

// ── TaskStore: transitions ────────────────────────────────────────────────

#[test]
fn store_valid_transition_updates_state_and_records_history() {
    let (store, _) = stores();
    let trace_id = TraceId::new();

    let task = pure_reason_runtime::types::Task::new(trace_id, WorkflowKind::Reasoning, "t");
    let task_id = task.task_id;
    store.create_task(&task).unwrap();

    let tr = store
        .transition(&task_id, TaskState::Executing, Some("starting"))
        .unwrap();
    assert_eq!(tr.from_state, TaskState::Created);
    assert_eq!(tr.to_state, TaskState::Executing);
    assert_eq!(tr.reason.as_deref(), Some("starting"));

    let loaded = store.load_task(&task_id).unwrap();
    assert_eq!(loaded.state, TaskState::Executing);
}

#[test]
fn store_invalid_transition_is_rejected() {
    let (store, _) = stores();
    let trace_id = TraceId::new();

    let task = pure_reason_runtime::types::Task::new(trace_id, WorkflowKind::Reasoning, "t");
    let task_id = task.task_id;
    store.create_task(&task).unwrap();

    // Created → Completed is not a valid edge.
    let err = store
        .transition(&task_id, TaskState::Completed, None)
        .unwrap_err();
    assert!(matches!(err, RuntimeError::InvalidTransition { .. }));
}

#[test]
fn store_transition_history_ordered_oldest_first() {
    let (store, _) = stores();
    let trace_id = TraceId::new();

    let task = pure_reason_runtime::types::Task::new(trace_id, WorkflowKind::Reasoning, "t");
    let task_id = task.task_id;
    store.create_task(&task).unwrap();

    store
        .transition(&task_id, TaskState::Executing, None)
        .unwrap();
    store
        .transition(&task_id, TaskState::Completed, None)
        .unwrap();

    let history = store.list_transitions(&task_id).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].from_state, TaskState::Created);
    assert_eq!(history[0].to_state, TaskState::Executing);
    assert_eq!(history[1].from_state, TaskState::Executing);
    assert_eq!(history[1].to_state, TaskState::Completed);
}

#[test]
fn store_list_recent_respects_limit() {
    let (store, _) = stores();
    let trace_id = TraceId::new();

    for i in 0..5 {
        let task = pure_reason_runtime::types::Task::new(
            trace_id,
            WorkflowKind::Reasoning,
            format!("task {i}"),
        );
        store.create_task(&task).unwrap();
    }

    let recent = store.list_recent(3).unwrap();
    assert_eq!(recent.len(), 3);
}

// ── Executor: completion path ─────────────────────────────────────────────

#[test]
fn executor_completion_path() {
    let exec = executor();
    let trace_id = TraceId::new();

    let task = exec
        .create_task(trace_id, WorkflowKind::Reasoning, "reason about X")
        .unwrap();
    let final_state = exec.run(&task.task_id, || StepOutcome::Completed).unwrap();

    assert_eq!(final_state, TaskState::Completed);
}

#[test]
fn executor_begin_and_finish_allow_external_persistence_between_steps() {
    let exec = executor();
    let trace_id = TraceId::new();

    let task = exec
        .create_task(
            trace_id,
            WorkflowKind::Verification,
            "persist evidence before finish",
        )
        .unwrap();

    let intermediate = exec.begin(&task.task_id).unwrap();
    assert_eq!(intermediate, TaskState::Executing);

    let final_state = exec
        .finish(&task.task_id, StepOutcome::NeedsReview)
        .unwrap();
    assert_eq!(final_state, TaskState::AwaitingReview);
}

// ── Executor: review path ─────────────────────────────────────────────────

#[test]
fn executor_review_path() {
    let exec = executor();
    let trace_id = TraceId::new();

    let task = exec
        .create_task(trace_id, WorkflowKind::CodeGen, "generate code")
        .unwrap();
    let final_state = exec
        .run(&task.task_id, || StepOutcome::NeedsReview)
        .unwrap();

    assert_eq!(final_state, TaskState::AwaitingReview);
}

// ── Executor: failure path ────────────────────────────────────────────────

#[test]
fn executor_failure_path() {
    let exec = executor();
    let trace_id = TraceId::new();

    let task = exec
        .create_task(trace_id, WorkflowKind::Verification, "verify")
        .unwrap();
    let final_state = exec
        .run(&task.task_id, || StepOutcome::Error("step exploded".into()))
        .unwrap();

    assert_eq!(final_state, TaskState::Failed);
}

// ── Executor: trace emission ──────────────────────────────────────────────

#[test]
fn executor_emits_trace_events_on_create_and_run() {
    // Use file-backed stores so we can re-open and inspect after the executor runs.
    let dir = tempfile::tempdir().unwrap();
    let task_db = dir.path().join("tasks.db");
    let trace_db = dir.path().join("trace.db");

    let exec = WorkflowExecutor::new(
        TaskStore::open(&task_db).unwrap(),
        TraceStore::open(&trace_db).unwrap(),
    );

    let trace_id = TraceId::new();
    let task = exec
        .create_task(trace_id, WorkflowKind::Summarisation, "summarise")
        .unwrap();
    exec.run(&task.task_id, || StepOutcome::Completed).unwrap();

    // Re-open the trace store and inspect events.
    let trace_store = TraceStore::open(&trace_db).unwrap();
    let events = trace_store.list_by_trace(&trace_id, 100).unwrap();

    // Expect: Created event + Executing event + Completed event = 3
    assert_eq!(
        events.len(),
        3,
        "expected 3 trace events, got {}",
        events.len()
    );

    use pure_reason_trace::types::TraceEventKind;
    assert!(events
        .iter()
        .all(|e| e.kind == TraceEventKind::TaskStateChange));
}

// ── TaskState: transition graph ───────────────────────────────────────────

#[test]
fn task_state_terminal_states_reject_all_transitions() {
    let terminals = [TaskState::Completed, TaskState::Failed, TaskState::TimedOut];
    let all_states = [
        TaskState::Created,
        TaskState::Planning,
        TaskState::Executing,
        TaskState::Verifying,
        TaskState::AwaitingReview,
        TaskState::Completed,
        TaskState::Failed,
        TaskState::TimedOut,
    ];
    for terminal in terminals {
        for target in all_states {
            assert!(
                !terminal.can_transition_to(target),
                "{terminal} should not transition to {target}"
            );
        }
    }
}

#[test]
fn task_state_created_can_reach_planning_and_executing() {
    assert!(TaskState::Created.can_transition_to(TaskState::Planning));
    assert!(TaskState::Created.can_transition_to(TaskState::Executing));
    assert!(!TaskState::Created.can_transition_to(TaskState::Completed));
    assert!(!TaskState::Created.can_transition_to(TaskState::AwaitingReview));
}
