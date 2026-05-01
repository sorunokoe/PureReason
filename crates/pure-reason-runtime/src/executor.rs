//! Minimal workflow executor: creates tasks, runs a caller-supplied step
//! closure, drives validated state transitions, and emits trace events.

use crate::{
    error::{Result, RuntimeError},
    store::TaskStore,
    types::{Task, TaskState, WorkflowKind},
};
use pure_reason_trace::{
    ids::{TaskId, TraceId},
    types::{TraceEvent, TraceEventPayload},
    TraceStore,
};

/// Outcome returned by the caller-supplied step runner.
pub enum StepOutcome {
    /// The task finished successfully.
    Completed,
    /// The task produced output that requires human or agent review.
    NeedsReview,
    /// The step encountered an unrecoverable error.
    Error(String),
}

impl StepOutcome {
    pub fn terminal_state(&self) -> TaskState {
        match self {
            Self::Completed => TaskState::Completed,
            Self::NeedsReview => TaskState::AwaitingReview,
            Self::Error(_) => TaskState::Failed,
        }
    }

    fn reason(&self) -> Option<&str> {
        match self {
            Self::Error(message) => Some(message.as_str()),
            _ => None,
        }
    }
}

/// Drives task lifecycle for a single workflow step.
///
/// The executor is deliberately minimal: it owns a `TaskStore` and a
/// `TraceStore`, creates tasks, runs a caller-supplied closure, and performs
/// the matching validated state transitions while emitting trace events.
pub struct WorkflowExecutor {
    tasks: TaskStore,
    traces: TraceStore,
}

impl WorkflowExecutor {
    /// Create an executor backed by existing stores.
    pub fn new(tasks: TaskStore, traces: TraceStore) -> Self {
        Self { tasks, traces }
    }

    /// Create and persist a new task, emitting a `TaskStateChange` trace event.
    pub fn create_task(
        &self,
        trace_id: TraceId,
        kind: WorkflowKind,
        description: impl Into<String>,
    ) -> Result<Task> {
        let task = Task::new(trace_id, kind, description);
        self.tasks.create_task(&task)?;
        self.emit_state_change(trace_id, task.task_id, "none", TaskState::Created)?;
        Ok(task)
    }

    /// Run `step_fn` against `task_id`, driving state transitions and emitting
    /// trace events.
    ///
    /// Transition path:
    /// - `Created` → `Executing` before the step runs
    /// - `Executing` → `Completed` when `step_fn` returns [`StepOutcome::Completed`]
    /// - `Executing` → `AwaitingReview` when it returns [`StepOutcome::NeedsReview`]
    /// - `Executing` → `Failed` when it returns [`StepOutcome::Error`]
    ///
    /// Returns the final `TaskState`.
    pub fn run<F>(&self, task_id: &TaskId, step_fn: F) -> Result<TaskState>
    where
        F: FnOnce() -> StepOutcome,
    {
        self.begin(task_id)?;
        let outcome = step_fn();
        self.finish(task_id, outcome)
    }

    /// Transition a created task into `Executing`.
    pub fn begin(&self, task_id: &TaskId) -> Result<TaskState> {
        let task = self.tasks.load_task(task_id)?;
        let trace_id = task.trace_id;
        let t = self.tasks.transition(task_id, TaskState::Executing, None)?;
        self.emit_state_change(
            trace_id,
            *task_id,
            &t.from_state.to_string(),
            TaskState::Executing,
        )?;
        Ok(TaskState::Executing)
    }

    /// Finish an executing task with the provided terminal outcome.
    pub fn finish(&self, task_id: &TaskId, outcome: StepOutcome) -> Result<TaskState> {
        let task = self.tasks.load_task(task_id)?;
        let trace_id = task.trace_id;
        let to_state = outcome.terminal_state();
        let reason = outcome.reason().map(str::to_string);
        let transition = self
            .tasks
            .transition(task_id, to_state, reason.as_deref())?;
        self.emit_state_change(
            trace_id,
            *task_id,
            &transition.from_state.to_string(),
            to_state,
        )?;
        Ok(to_state)
    }

    // ── Helpers ────────────────────────────────────────────────────────────

    fn emit_state_change(
        &self,
        trace_id: TraceId,
        task_id: TaskId,
        from_state: &str,
        to_state: TaskState,
    ) -> Result<()> {
        let event = TraceEvent::new(
            trace_id,
            Some(task_id),
            TraceEventPayload::TaskStateChange {
                task_id,
                from_state: from_state.to_string(),
                to_state: to_state.to_string(),
            },
        );
        self.traces.append(&event).map_err(RuntimeError::Trace)
    }
}
