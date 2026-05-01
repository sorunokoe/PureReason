//! # pure-reason-runtime
//!
//! Task state machine, workflow executor, and trace-wired runtime skeleton
//! for PureReason.
//!
//! ## Crate layout
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`types`] | `WorkflowKind`, `TaskState`, `Task`, `StateTransition` |
//! | [`store`] | SQLite-backed `TaskStore` (create / load / transition / list) |
//! | [`executor`] | `WorkflowExecutor` — closure-driven step runner with trace emission |
//! | [`local_state`] | Zero-config local state paths and durable task/trace store opening |
//! | [`error`] | `RuntimeError` and `Result` alias |
//!
//! ## Usage sketch
//!
//! ```rust,no_run
//! use pure_reason_runtime::{
//!     executor::{StepOutcome, WorkflowExecutor},
//!     store::TaskStore,
//!     types::WorkflowKind,
//! };
//! use pure_reason_trace::{ids::TraceId, TraceStore};
//!
//! let tasks  = TaskStore::open_in_memory().unwrap();
//! let traces = TraceStore::open_in_memory().unwrap();
//! let exec   = WorkflowExecutor::new(tasks, traces);
//!
//! let trace_id = TraceId::new();
//! let task = exec.create_task(trace_id, WorkflowKind::Reasoning, "Explain AGI").unwrap();
//! let final_state = exec.run(&task.task_id, || StepOutcome::Completed).unwrap();
//! ```

pub mod error;
pub mod executor;
pub mod local_state;
pub mod store;
pub mod types;

pub use error::{Result, RuntimeError};
pub use local_state::{
    default_local_state_dir, open_local_stores, resolve_local_state_dir, LocalStatePaths,
    DEFAULT_STATE_DIR_NAME, STATE_DIR_ENV, TASK_DB_FILE_NAME, TRACE_DB_FILE_NAME,
};
pub use types::{StateTransition, Task, TaskState, WorkflowKind};
