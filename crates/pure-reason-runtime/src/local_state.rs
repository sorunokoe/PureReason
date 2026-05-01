use crate::{store::TaskStore, Result};
use pure_reason_trace::TraceStore;
use std::path::PathBuf;

pub const STATE_DIR_ENV: &str = "PURE_REASON_STATE_DIR";
pub const DEFAULT_STATE_DIR_NAME: &str = "agent-state";
pub const TASK_DB_FILE_NAME: &str = "tasks.sqlite3";
pub const TRACE_DB_FILE_NAME: &str = "traces.sqlite3";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalStatePaths {
    pub dir: PathBuf,
    pub task_db: PathBuf,
    pub trace_db: PathBuf,
}

impl LocalStatePaths {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        Self {
            task_db: dir.join(TASK_DB_FILE_NAME),
            trace_db: dir.join(TRACE_DB_FILE_NAME),
            dir,
        }
    }
}

pub fn default_local_state_dir() -> Result<PathBuf> {
    Ok(home_dir()?
        .join(".pure-reason")
        .join(DEFAULT_STATE_DIR_NAME))
}

pub fn resolve_local_state_dir(override_dir: Option<PathBuf>) -> Result<PathBuf> {
    resolve_local_state_dir_inner(
        override_dir,
        std::env::var_os(STATE_DIR_ENV)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from),
        home_dir().ok(),
    )
}

pub fn open_local_stores(dir: impl Into<PathBuf>) -> Result<(TaskStore, TraceStore)> {
    let paths = LocalStatePaths::new(dir);
    let task_store = TaskStore::open(&paths.task_db)?;
    let trace_store = TraceStore::open(&paths.trace_db)?;
    Ok((task_store, trace_store))
}

fn resolve_local_state_dir_inner(
    override_dir: Option<PathBuf>,
    env_dir: Option<PathBuf>,
    home_dir: Option<PathBuf>,
) -> Result<PathBuf> {
    if let Some(dir) = override_dir.or(env_dir) {
        return Ok(dir);
    }

    let home_dir = home_dir.ok_or_else(|| {
        crate::error::RuntimeError::Configuration(
            "unable to resolve a home directory; set PURE_REASON_STATE_DIR explicitly".to_string(),
        )
    })?;

    Ok(home_dir.join(".pure-reason").join(DEFAULT_STATE_DIR_NAME))
}

fn home_dir() -> Result<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .map_err(|_| {
            crate::error::RuntimeError::Configuration(
                "unable to resolve a home directory; set PURE_REASON_STATE_DIR explicitly"
                    .to_string(),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        executor::{StepOutcome, WorkflowExecutor},
        WorkflowKind,
    };
    use pure_reason_trace::ids::TraceId;
    use tempfile::TempDir;

    #[test]
    fn local_state_paths_use_expected_file_names() {
        let paths = LocalStatePaths::new(PathBuf::from("/tmp/pure-reason-state"));
        assert_eq!(
            paths.task_db,
            PathBuf::from("/tmp/pure-reason-state/tasks.sqlite3")
        );
        assert_eq!(
            paths.trace_db,
            PathBuf::from("/tmp/pure-reason-state/traces.sqlite3")
        );
    }

    #[test]
    fn override_dir_wins_over_env_dir() {
        let resolved = resolve_local_state_dir_inner(
            Some(PathBuf::from("/tmp/override")),
            Some(PathBuf::from("/tmp/env")),
            None,
        )
        .unwrap();
        assert_eq!(resolved, PathBuf::from("/tmp/override"));
    }

    #[test]
    fn env_dir_is_used_when_no_override_is_present() {
        let resolved =
            resolve_local_state_dir_inner(None, Some(PathBuf::from("/tmp/env")), None).unwrap();
        assert_eq!(resolved, PathBuf::from("/tmp/env"));
    }

    #[test]
    fn missing_home_dir_requires_explicit_state_dir() {
        let error = resolve_local_state_dir_inner(None, None, None).unwrap_err();
        assert!(matches!(
            error,
            crate::error::RuntimeError::Configuration(_)
        ));
    }

    #[test]
    fn local_stores_persist_tasks_and_traces_across_reopen() {
        let temp_dir = TempDir::new().unwrap();
        let state_dir = temp_dir.path().join("agent-state");

        {
            let (task_store, trace_store) = open_local_stores(&state_dir).unwrap();
            let executor = WorkflowExecutor::new(task_store, trace_store.clone());
            let trace_id = TraceId::new();
            let task = executor
                .create_task(trace_id, WorkflowKind::Verification, "Persisted review")
                .unwrap();

            executor
                .run(&task.task_id, || StepOutcome::Completed)
                .unwrap();
        }

        let (task_store, trace_store) = open_local_stores(&state_dir).unwrap();
        let tasks = task_store.list_recent(10).unwrap();
        assert_eq!(tasks.len(), 1);

        let events = trace_store.list_by_trace(&tasks[0].trace_id, 10).unwrap();
        assert_eq!(events.len(), 3);
    }
}
