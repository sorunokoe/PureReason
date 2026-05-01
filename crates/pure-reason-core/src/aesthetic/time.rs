//! # Time — The Form of Inner Intuition
//!
//! For Kant, **Time** is the pure form of inner sense. It is the a priori condition
//! for all inner experience, and mediates all outer experience as well.
//! "Time is the formal a priori condition of all appearances whatsoever." (CPR A34/B50)
//!
//! Unlike Space (which is the form of outer sense), Time is the form of inner sense:
//! the way we experience our own states in sequence.
//!
//! In this system, TimeForm captures the **sequential and temporal organization**
//! of text: the order of events, temporal references (before/after/now), durations,
//! and the structure of the context window or conversation history.

use super::Manifold;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── TemporalMarker ──────────────────────────────────────────────────────────

/// A detected temporal reference in text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalMarker {
    pub id: Uuid,
    pub text: String,
    pub position: usize,
    pub kind: TemporalMarkerKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporalMarkerKind {
    /// References to the past (e.g., "yesterday", "before", "previously")
    Past,
    /// References to the present (e.g., "now", "currently", "today")
    Present,
    /// References to the future (e.g., "tomorrow", "will", "shall")
    Future,
    /// Duration expressions (e.g., "for 3 hours", "since", "until")
    Duration,
    /// Ordering expressions (e.g., "first", "then", "finally", "after")
    Sequential,
}

// ─── TemporalEvent ───────────────────────────────────────────────────────────

/// A detected event or state in the text, placed in temporal sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalEvent {
    pub id: Uuid,
    pub description: String,
    /// Position in the source token sequence.
    pub token_position: usize,
    /// Inferred temporal position (0.0 = beginning, 1.0 = end of described time).
    pub relative_position: f64,
    pub tense: Tense,
}

/// Grammatical tense of an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tense {
    Past,
    Present,
    Future,
    Unknown,
}

impl TemporalEvent {
    pub fn new(description: impl Into<String>, token_position: usize, tense: Tense) -> Self {
        Self {
            id: Uuid::new_v4(),
            description: description.into(),
            token_position,
            relative_position: 0.0,
            tense,
        }
    }
}

// ─── TemporalOrder ───────────────────────────────────────────────────────────

/// A before/after ordering relation between two events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalOrder {
    pub before: Uuid,
    pub after: Uuid,
    pub relation: OrderRelation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderRelation {
    /// A immediately precedes B
    Immediately,
    /// A precedes B (possibly with gap)
    Before,
    /// A and B are simultaneous
    Simultaneous,
    /// A follows B
    After,
}

// ─── TimeForm ────────────────────────────────────────────────────────────────

/// The Form of Time — the sequential and temporal organization of input text.
///
/// Captures the order of events, temporal markers, and the flow of time
/// as represented in the text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeForm {
    /// Detected temporal markers in the text.
    pub markers: Vec<TemporalMarker>,
    /// Detected events/states, ordered in time.
    pub events: Vec<TemporalEvent>,
    /// Ordering relations between events.
    pub orderings: Vec<TemporalOrder>,
    /// The overall temporal orientation of the text.
    pub orientation: TemporalOrientation,
}

/// The dominant temporal orientation of a text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporalOrientation {
    PrimarilyPast,
    PrimarilyPresent,
    PrimarilyFuture,
    Mixed,
    Atemporal,
}

impl TimeForm {
    /// Organize a manifold into a temporal form.
    pub fn organize(manifold: &Manifold) -> Result<Self> {
        let markers = detect_temporal_markers(&manifold.tokens);
        let events = extract_events(&manifold.sentences);
        let orderings = infer_orderings(&events);
        let orientation = determine_orientation(&markers);
        Ok(Self {
            markers,
            events,
            orderings,
            orientation,
        })
    }

    /// Get events in chronological order (by relative_position).
    pub fn chronological_events(&self) -> Vec<&TemporalEvent> {
        let mut sorted: Vec<&TemporalEvent> = self.events.iter().collect();
        sorted.sort_by(|a, b| a.relative_position.total_cmp(&b.relative_position));
        sorted
    }
}

// ─── Heuristic Implementations ───────────────────────────────────────────────

const PAST_MARKERS: &[&str] = &[
    "yesterday",
    "before",
    "previously",
    "ago",
    "was",
    "were",
    "had",
    "did",
    "earlier",
    "once",
    "formerly",
    "past",
    "history",
    "already",
];

const PRESENT_MARKERS: &[&str] = &[
    "now",
    "currently",
    "today",
    "is",
    "are",
    "am",
    "being",
    "present",
    "nowadays",
    "still",
    "ongoing",
];

const FUTURE_MARKERS: &[&str] = &[
    "tomorrow",
    "will",
    "shall",
    "soon",
    "later",
    "next",
    "future",
    "eventually",
    "upcoming",
    "going to",
    "would",
];

const SEQUENTIAL_MARKERS: &[&str] = &[
    "first",
    "second",
    "third",
    "then",
    "after",
    "before",
    "finally",
    "next",
    "subsequently",
    "previously",
    "initially",
    "last",
];

fn detect_temporal_markers(tokens: &[String]) -> Vec<TemporalMarker> {
    tokens
        .iter()
        .enumerate()
        .filter_map(|(i, t)| {
            let kind = if PAST_MARKERS.contains(&t.as_str()) {
                Some(TemporalMarkerKind::Past)
            } else if PRESENT_MARKERS.contains(&t.as_str()) {
                Some(TemporalMarkerKind::Present)
            } else if FUTURE_MARKERS.contains(&t.as_str()) {
                Some(TemporalMarkerKind::Future)
            } else if SEQUENTIAL_MARKERS.contains(&t.as_str()) {
                Some(TemporalMarkerKind::Sequential)
            } else {
                None
            };
            kind.map(|k| TemporalMarker {
                id: Uuid::new_v4(),
                text: t.clone(),
                position: i,
                kind: k,
            })
        })
        .collect()
}

fn infer_tense(sentence: &str) -> Tense {
    let lower = sentence.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    let past_count = words.iter().filter(|&&w| PAST_MARKERS.contains(&w)).count();
    let present_count = words
        .iter()
        .filter(|&&w| PRESENT_MARKERS.contains(&w))
        .count();
    let future_count = words
        .iter()
        .filter(|&&w| FUTURE_MARKERS.contains(&w))
        .count();

    if past_count > present_count && past_count > future_count {
        Tense::Past
    } else if future_count > present_count && future_count > past_count {
        Tense::Future
    } else if present_count > 0 {
        Tense::Present
    } else {
        Tense::Unknown
    }
}

fn extract_events(sentences: &[String]) -> Vec<TemporalEvent> {
    sentences
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let tense = infer_tense(s);
            let mut event = TemporalEvent::new(s.clone(), i, tense);
            // Assign relative position based on sentence index
            event.relative_position = i as f64;
            event
        })
        .collect()
}

fn infer_orderings(events: &[TemporalEvent]) -> Vec<TemporalOrder> {
    events
        .windows(2)
        .map(|pair| TemporalOrder {
            before: pair[0].id,
            after: pair[1].id,
            relation: OrderRelation::Before,
        })
        .collect()
}

fn determine_orientation(markers: &[TemporalMarker]) -> TemporalOrientation {
    if markers.is_empty() {
        return TemporalOrientation::Atemporal;
    }

    let past = markers
        .iter()
        .filter(|m| m.kind == TemporalMarkerKind::Past)
        .count();
    let present = markers
        .iter()
        .filter(|m| m.kind == TemporalMarkerKind::Present)
        .count();
    let future = markers
        .iter()
        .filter(|m| m.kind == TemporalMarkerKind::Future)
        .count();

    let max = past.max(present).max(future);
    if max == 0 {
        return TemporalOrientation::Atemporal;
    }

    let dominant_count = [past, present, future]
        .iter()
        .filter(|&&c| c == max)
        .count();
    if dominant_count > 1 {
        TemporalOrientation::Mixed
    } else if past == max {
        TemporalOrientation::PrimarilyPast
    } else if future == max {
        TemporalOrientation::PrimarilyFuture
    } else {
        TemporalOrientation::PrimarilyPresent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aesthetic::Manifold;

    #[test]
    fn time_form_detects_past_markers() {
        let m = Manifold::from_text("Yesterday it was raining before noon.");
        let tf = TimeForm::organize(&m).unwrap();
        let past = tf
            .markers
            .iter()
            .filter(|m| m.kind == TemporalMarkerKind::Past)
            .count();
        assert!(past > 0, "Should detect past markers");
    }

    #[test]
    fn time_form_detects_future_markers() {
        let m = Manifold::from_text("Tomorrow it will rain and shall be cold.");
        let tf = TimeForm::organize(&m).unwrap();
        let future = tf
            .markers
            .iter()
            .filter(|m| m.kind == TemporalMarkerKind::Future)
            .count();
        assert!(future > 0);
    }

    #[test]
    fn chronological_events_sorted() {
        let m = Manifold::from_text("First this. Then that. Finally the other.");
        let tf = TimeForm::organize(&m).unwrap();
        let events = tf.chronological_events();
        for pair in events.windows(2) {
            assert!(pair[0].relative_position <= pair[1].relative_position);
        }
    }
}
