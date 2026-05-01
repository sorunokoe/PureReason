//! # FeedbackCollector (TRIZ A-2)
//!
//! Collects user corrections to missed/false detections, persists them to
//! `~/.pure-reason/feedback.jsonl`, and proposes new signal phrases via
//! `suggest_training()`.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ─── FeedbackKind ────────────────────────────────────────────────────────────

/// The kind of feedback event being recorded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FeedbackKind {
    /// A detection was missed — add signal phrase.
    MissedIllusion {
        /// The illusion/antinomy kind (free-text for extensibility).
        kind: String,
        /// The phrase the pipeline should have caught.
        phrase: String,
    },
    /// A false positive — the detection was wrong.
    FalsePositive {
        /// The kind that was incorrectly flagged.
        kind: String,
        /// The phrase that triggered the false positive.
        phrase: String,
    },
    /// The risk level assessment was wrong.
    WrongRiskLevel {
        /// What the correct risk level should have been.
        expected: String,
        /// What the pipeline returned.
        got: String,
    },
}

// ─── FeedbackEvent ───────────────────────────────────────────────────────────

/// A single feedback event written to the JSONL store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackEvent {
    /// ISO-8601 timestamp.
    pub timestamp: String,
    /// The input text that triggered (or failed to trigger) the detection.
    pub input: String,
    /// The feedback kind.
    pub correction: FeedbackKind,
    /// Optional free-text notes.
    pub notes: Option<String>,
}

impl FeedbackEvent {
    pub fn new(input: impl Into<String>, correction: FeedbackKind) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            input: input.into(),
            correction,
            notes: None,
        }
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

// ─── FeedbackCollector ───────────────────────────────────────────────────────

/// Collects and persists user corrections to epistemic detection.
pub struct FeedbackCollector {
    path: PathBuf,
}

impl FeedbackCollector {
    /// Create a collector using the default path `~/.pure-reason/feedback.jsonl`.
    pub fn new() -> Self {
        let path = home_dir().join(".pure-reason").join("feedback.jsonl");
        Self { path }
    }

    /// Create a collector with a custom path (useful for testing).
    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }

    /// Record a feedback event, appending it to the JSONL store.
    pub fn record(&self, event: &FeedbackEvent) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let line = serde_json::to_string(event)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    /// Load all feedback events from the JSONL store.
    pub fn load_all(&self) -> Result<Vec<FeedbackEvent>> {
        if !self.path.exists() {
            return Ok(vec![]);
        }
        let content = std::fs::read_to_string(&self.path)?;
        let events = content
            .lines()
            .filter(|l| !l.is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
        Ok(events)
    }

    /// Suggest signal phrases to add, based on frequently missed illusions.
    ///
    /// Returns a list of human-readable suggestion strings. Phrases that appear
    /// in ≥2 `MissedIllusion` events are included.
    pub fn suggest_training(&self) -> Result<Vec<TrainingSuggestion>> {
        let events = self.load_all()?;

        let mut missed: std::collections::HashMap<(String, String), usize> = Default::default();
        let mut false_pos: std::collections::HashMap<(String, String), usize> = Default::default();

        for event in &events {
            match &event.correction {
                FeedbackKind::MissedIllusion { kind, phrase } => {
                    *missed.entry((kind.clone(), phrase.clone())).or_insert(0) += 1;
                }
                FeedbackKind::FalsePositive { kind, phrase } => {
                    *false_pos.entry((kind.clone(), phrase.clone())).or_insert(0) += 1;
                }
                _ => {}
            }
        }

        let mut suggestions = Vec::new();

        for ((kind, phrase), count) in missed {
            if count >= 2 {
                suggestions.push(TrainingSuggestion {
                    action: TrainingAction::AddSignal,
                    kind,
                    phrase,
                    occurrences: count,
                });
            }
        }

        for ((kind, phrase), count) in false_pos {
            if count >= 2 {
                suggestions.push(TrainingSuggestion {
                    action: TrainingAction::RemoveSignal,
                    kind,
                    phrase,
                    occurrences: count,
                });
            }
        }

        suggestions.sort_by_key(|s| std::cmp::Reverse(s.occurrences));
        Ok(suggestions)
    }

    /// Return the path to the feedback store.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Total number of recorded events.
    pub fn event_count(&self) -> Result<usize> {
        Ok(self.load_all()?.len())
    }
}

impl Default for FeedbackCollector {
    fn default() -> Self {
        Self::new()
    }
}

// ─── TrainingSuggestion ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSuggestion {
    pub action: TrainingAction,
    pub kind: String,
    pub phrase: String,
    pub occurrences: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrainingAction {
    /// Add phrase to detection signals for this illusion kind.
    AddSignal,
    /// Remove phrase from detection signals (too many false positives).
    RemoveSignal,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_collector() -> (FeedbackCollector, TempDir) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("feedback.jsonl");
        (FeedbackCollector::with_path(path), dir)
    }

    #[test]
    fn record_and_load() {
        let (collector, _dir) = temp_collector();
        let event = FeedbackEvent::new(
            "God exists necessarily",
            FeedbackKind::MissedIllusion {
                kind: "Theological".to_string(),
                phrase: "god exists necessarily".to_string(),
            },
        );
        collector.record(&event).unwrap();
        let loaded = collector.load_all().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].input, "God exists necessarily");
    }

    #[test]
    fn suggest_training_threshold() {
        let (collector, _dir) = temp_collector();
        for _ in 0..3 {
            let event = FeedbackEvent::new(
                "The first cause must exist",
                FeedbackKind::MissedIllusion {
                    kind: "Theological".to_string(),
                    phrase: "first cause must exist".to_string(),
                },
            );
            collector.record(&event).unwrap();
        }
        let suggestions = collector.suggest_training().unwrap();
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].phrase, "first cause must exist");
    }

    #[test]
    fn empty_feedback_no_suggestions() {
        let (collector, _dir) = temp_collector();
        let suggestions = collector.suggest_training().unwrap();
        assert!(suggestions.is_empty());
    }
}
