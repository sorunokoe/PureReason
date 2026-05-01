//! Core event types for the trace subsystem.

use crate::ids::{EventId, TaskId, TraceId};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// High-level category of a trace event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceEventKind {
    /// An LLM / model inference call.
    ModelCall,
    /// A task moved between lifecycle states.
    TaskStateChange,
    /// The output of a verification step.
    VerificationResult,
    /// A policy or routing decision was made.
    Decision,
    /// A free-form diagnostic note.
    Note,
}

impl std::fmt::Display for TraceEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::ModelCall => "model_call",
            Self::TaskStateChange => "task_state_change",
            Self::VerificationResult => "verification_result",
            Self::Decision => "decision",
            Self::Note => "note",
        };
        write!(f, "{s}")
    }
}

/// Typed payload for each event kind.
///
/// Adding new variants is backward-compatible because the store uses a JSON
/// blob; old readers will deserialize into `Note` or skip unknown payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TraceEventPayload {
    /// An LLM inference call with basic telemetry.
    ModelCall {
        /// The model identifier reported by the provider.
        model: String,
        /// The provider that handled this call (e.g. "openai", "anthropic").
        provider: String,
        /// Estimated or actual prompt tokens consumed.
        prompt_tokens: u32,
        /// Completion tokens produced.
        completion_tokens: u32,
        /// Wall-clock latency of the provider call.
        latency_ms: u64,
        /// "success" or "error".
        status: String,
    },
    /// A task transitioned from one state to another.
    TaskStateChange {
        task_id: TaskId,
        from_state: String,
        to_state: String,
    },
    /// The outcome of a verification / validation step.
    VerificationResult {
        passed: bool,
        /// Normalized score in [0, 1].
        score: f64,
        details: String,
    },
    /// A policy or routing decision.
    Decision { action: String, rationale: String },
    /// A free-form diagnostic note.
    Note { message: String },
}

impl TraceEventPayload {
    /// Return the [`TraceEventKind`] that corresponds to this payload.
    pub fn kind(&self) -> TraceEventKind {
        match self {
            Self::ModelCall { .. } => TraceEventKind::ModelCall,
            Self::TaskStateChange { .. } => TraceEventKind::TaskStateChange,
            Self::VerificationResult { .. } => TraceEventKind::VerificationResult,
            Self::Decision { .. } => TraceEventKind::Decision,
            Self::Note { .. } => TraceEventKind::Note,
        }
    }
}

/// An immutable event record in the trace store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Unique identifier for this event.
    pub event_id: EventId,
    /// The top-level trace this event belongs to.
    pub trace_id: TraceId,
    /// Optional sub-task within the trace.
    pub task_id: Option<TaskId>,
    /// High-level event category (denormalised for indexing).
    pub kind: TraceEventKind,
    /// RFC 3339 timestamp set at creation time.
    pub timestamp: String,
    /// Typed payload carrying the event-specific data.
    pub payload: TraceEventPayload,
}

impl TraceEvent {
    /// Create a new event with the current UTC timestamp.
    pub fn new(trace_id: TraceId, task_id: Option<TaskId>, payload: TraceEventPayload) -> Self {
        let kind = payload.kind();
        Self {
            event_id: EventId::new(),
            trace_id,
            task_id,
            kind,
            timestamp: Utc::now().to_rfc3339(),
            payload,
        }
    }
}
