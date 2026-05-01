//! # Persistent Agent Sessions (S-III-6)
//!
//! Maintains state across multiple pipeline invocations by persisting the
//! `WorldModel` and turn history to `~/.pure-reason/sessions/<id>.json`.
//!
//! This resolves the TC: *each call learns context* ↔ *zero cross-call memory*.
//! The IFR: the agent remembers what it needs without requiring an external
//! database — it uses resources already present (filesystem + serde_json).
//!
//! ## Usage
//! ```rust,ignore
//! let mut session = AgentSession::load("my-project").unwrap_or_else(|| AgentSession::new("my-project"));
//! session.record_turn("input text", "SAFE", false, 0);
//! session.save().unwrap();
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::world_model::WorldModel;

// ─── AgentSession ─────────────────────────────────────────────────────────────

/// A persistent agent session with accumulated WorldModel and turn history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    /// Unique session identifier (user-assigned slug).
    pub id: String,
    /// The accumulated world model for this session.
    pub world_model: WorldModel,
    /// Number of turns completed.
    pub turn_count: usize,
    /// ISO 8601 UTC timestamp when this session was created.
    pub created_at: String,
    /// ISO 8601 UTC timestamp of the last update.
    pub last_updated: String,
    /// Per-turn summary records.
    pub history: Vec<SessionTurn>,
}

/// A single-turn summary record in a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTurn {
    pub turn: usize,
    pub input_preview: String, // first 120 chars of input
    pub risk_level: String,
    pub regulated: bool,
    pub issues_count: usize,
}

impl AgentSession {
    /// Create a brand-new session with the given ID.
    pub fn new(id: &str) -> Self {
        let now = now_iso8601();
        Self {
            id: id.to_string(),
            world_model: WorldModel::default(),
            turn_count: 0,
            created_at: now.clone(),
            last_updated: now,
            history: Vec::new(),
        }
    }

    /// Load a session from disk, or return `None` if it does not exist.
    pub fn load(id: &str) -> Option<Self> {
        let path = Self::session_path(id).ok()?;
        let json = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&json).ok()
    }

    /// Persist the session to `~/.pure-reason/sessions/<id>.json`.
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::session_path(&self.id)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        // Atomic write: write to temp file then rename (POSIX-atomic).
        let tmp_path = path.with_extension("json.tmp");
        std::fs::write(&tmp_path, json)?;
        std::fs::rename(&tmp_path, &path)?;
        Ok(())
    }

    /// Record the outcome of one pipeline turn.
    pub fn record_turn(
        &mut self,
        input: &str,
        risk_level: &str,
        regulated: bool,
        issues_count: usize,
    ) {
        self.turn_count += 1;
        self.last_updated = now_iso8601();
        let preview = input.chars().take(120).collect::<String>();
        self.history.push(SessionTurn {
            turn: self.turn_count,
            input_preview: preview,
            risk_level: risk_level.to_string(),
            regulated,
            issues_count,
        });
    }

    /// List all session IDs found on disk.
    pub fn list_all() -> Vec<String> {
        let dir = sessions_dir();
        let Ok(entries) = std::fs::read_dir(&dir) else {
            return Vec::new();
        };
        entries
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                name.strip_suffix(".json").map(str::to_string)
            })
            .collect()
    }

    /// Delete a session from disk.
    pub fn delete(id: &str) -> std::io::Result<()> {
        let path = Self::session_path(id)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
        std::fs::remove_file(path)
    }

    fn session_path(id: &str) -> Result<PathBuf, crate::error::PureReasonError> {
        // Reject IDs containing path separators or leading dots to prevent traversal.
        if id.is_empty()
            || !id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(crate::error::PureReasonError::InvalidInput(
                "Session ID may only contain ASCII letters, digits, hyphens, and underscores"
                    .into(),
            ));
        }
        Ok(sessions_dir().join(format!("{}.json", id)))
    }
}

fn sessions_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".pure-reason").join("sessions")
}

fn now_iso8601() -> String {
    use chrono::Utc;
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_new_has_zero_turns() {
        let s = AgentSession::new("test-session");
        assert_eq!(s.turn_count, 0);
        assert!(s.history.is_empty());
    }

    #[test]
    fn session_record_turn_increments() {
        let mut s = AgentSession::new("t");
        s.record_turn("hello world", "SAFE", false, 0);
        assert_eq!(s.turn_count, 1);
        assert_eq!(s.history[0].risk_level, "SAFE");
        assert_eq!(s.history[0].turn, 1);
    }

    #[test]
    fn session_roundtrip_json() {
        let mut s = AgentSession::new("roundtrip");
        s.record_turn("Water boils at 100°C.", "SAFE", false, 0);
        let json = serde_json::to_string(&s).unwrap();
        let loaded: AgentSession = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.id, "roundtrip");
        assert_eq!(loaded.turn_count, 1);
    }
}
