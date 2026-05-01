//! Core runtime types: workflow kinds, task states, and history entries.

use chrono::Utc;
use pure_reason_trace::ids::{TaskId, TraceId};
use serde::{Deserialize, Serialize};

/// High-level category of work a task represents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowKind {
    /// General-purpose reasoning or analysis task.
    Reasoning,
    /// Code generation or modification task.
    CodeGen,
    /// Fact-checking or verification task.
    Verification,
    /// Summarisation or extraction task.
    Summarisation,
    /// Any other application-defined kind.
    Custom(String),
}

impl std::fmt::Display for WorkflowKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reasoning => write!(f, "reasoning"),
            Self::CodeGen => write!(f, "code_gen"),
            Self::Verification => write!(f, "verification"),
            Self::Summarisation => write!(f, "summarisation"),
            Self::Custom(s) => write!(f, "custom:{s}"),
        }
    }
}

/// Explicit lifecycle states for a runtime task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    /// Task has been created but execution has not started.
    Created,
    /// The executor is producing a plan before running steps.
    Planning,
    /// Active execution of steps is in progress.
    Executing,
    /// A verification pass is being run on the output.
    Verifying,
    /// Execution is paused, waiting for a human (or external agent) review.
    AwaitingReview,
    /// Task finished successfully.
    Completed,
    /// Task finished with an unrecoverable error.
    Failed,
    /// Task was forcibly stopped because it exceeded a time limit.
    TimedOut,
}

impl TaskState {
    /// Return `true` if no further transitions are allowed.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::TimedOut)
    }

    /// Return `true` if `to` is a valid successor of `self`.
    pub fn can_transition_to(self, to: TaskState) -> bool {
        use TaskState::*;
        matches!(
            (self, to),
            (Created, Planning)
                | (Created, Executing)
                | (Created, Failed)
                | (Planning, Executing)
                | (Planning, AwaitingReview)
                | (Planning, Failed)
                | (Executing, Verifying)
                | (Executing, AwaitingReview)
                | (Executing, Completed)
                | (Executing, Failed)
                | (Executing, TimedOut)
                | (Verifying, Completed)
                | (Verifying, AwaitingReview)
                | (Verifying, Failed)
                | (AwaitingReview, Executing)
                | (AwaitingReview, Completed)
                | (AwaitingReview, Failed)
        )
    }
}

impl std::fmt::Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Created => "created",
            Self::Planning => "planning",
            Self::Executing => "executing",
            Self::Verifying => "verifying",
            Self::AwaitingReview => "awaiting_review",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::TimedOut => "timed_out",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for TaskState {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "created" => Ok(Self::Created),
            "planning" => Ok(Self::Planning),
            "executing" => Ok(Self::Executing),
            "verifying" => Ok(Self::Verifying),
            "awaiting_review" => Ok(Self::AwaitingReview),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "timed_out" => Ok(Self::TimedOut),
            other => Err(format!("unknown TaskState: {other}")),
        }
    }
}

/// A runtime task and its current state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Stable identifier for this task.
    pub task_id: TaskId,
    /// The trace this task belongs to.
    pub trace_id: TraceId,
    /// What kind of work this task performs.
    pub kind: WorkflowKind,
    /// Current lifecycle state.
    pub state: TaskState,
    /// Short human-readable description of the task goal.
    pub description: String,
    /// RFC 3339 timestamp when the task was created.
    pub created_at: String,
    /// RFC 3339 timestamp of the most recent state change.
    pub updated_at: String,
}

impl Task {
    /// Create a new task in the `Created` state.
    pub fn new(trace_id: TraceId, kind: WorkflowKind, description: impl Into<String>) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            task_id: TaskId::new(),
            trace_id,
            kind,
            state: TaskState::Created,
            description: description.into(),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

/// A single recorded state transition for a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// The task this transition belongs to.
    pub task_id: TaskId,
    /// State before the transition.
    pub from_state: TaskState,
    /// State after the transition.
    pub to_state: TaskState,
    /// RFC 3339 timestamp of when the transition was recorded.
    pub timestamp: String,
    /// Optional human-readable reason for the transition.
    pub reason: Option<String>,
}
