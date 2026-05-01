//! # pure-reason-trace
//!
//! Append-only trace/event foundation for PureReason.
//!
//! Provides stable IDs, typed event payloads, and a SQLite-backed store that
//! later crates (`pure-reason-gateway`, `pure-reason-runtime`) can share as a
//! clean dependency without pulling in the full `pure-reason-core`.
//!
//! ## Storage layout
//!
//! `trace_events(event_id PK, trace_id, task_id, kind, timestamp, data)`
//! - indexed on `(trace_id, timestamp ASC)` for ordered per-trace queries
//! - indexed on `timestamp DESC` for recency queries
//! - JSON blob in `data` keeps schema forward-compatible

pub mod error;
pub mod ids;
pub mod store;
pub mod types;

pub use error::{Result, TraceError};
pub use ids::{EventId, EvidenceId, TaskId, TraceId};
pub use store::TraceStore;
pub use types::{TraceEvent, TraceEventKind, TraceEventPayload};
