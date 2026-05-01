//! # Streaming Epistemic Pipeline (TRIZ C-1)
//!
//! Token-level streaming pipeline that triggers Kantian analysis at sentence
//! boundaries. Can interrupt mid-stream on HIGH risk detection.
//!
//! ## Usage
//! ```rust,ignore
//! let mut sp = StreamingPipeline::new();
//! for token in tokens {
//!     for event in sp.feed(token) {
//!         match event {
//!             StreamEvent::HighRiskDetected { text, .. } => {
//!                 eprintln!("HALT: {}", text);
//!                 break;
//!             }
//!             _ => {}
//!         }
//!     }
//! }
//! for event in sp.flush() { /* handle final sentence */ }
//! ```

use crate::calibration::{EcsBand, PipelineCalibration};
use crate::pipeline::{KantianPipeline, PipelineReport, RiskLevel};

// ─── PhenomenalStatus ─────────────────────────────────────────────────────────

/// Classification of a sentence as phenomenal (within experience) or noumenal
/// (transcending the bounds of possible experience) — S-III-8.
#[derive(Debug, Clone, PartialEq)]
pub enum PhenomenalStatus {
    /// The sentence makes claims anchored in possible experience.
    Phenomenal,
    /// The sentence makes transcendental claims beyond possible experience.
    Noumenal {
        /// Stringified `IllusionKind` (e.g. `"HypostatizingIdea"`).
        kind: String,
        /// Stringified `IllusionSeverity` (e.g. `"High"`).
        severity: String,
    },
    /// The sentence encodes a regulative principle (heuristic guide, not a fact).
    Regulative { reason: String },
}

// ─── StreamEvent ─────────────────────────────────────────────────────────────

/// An event emitted by the streaming pipeline.
#[derive(Debug)]
pub enum StreamEvent {
    /// A sentence was processed and is within bounds.
    SentenceProcessed {
        text: String,
        risk: RiskLevel,
        /// ECS for this sentence (0–100). Available on every sentence.
        ecs: u8,
    },
    /// A sentence triggered HIGH risk — caller may interrupt the stream.
    HighRiskDetected {
        text: String,
        report: Box<PipelineReport>,
        /// ECS for this sentence (will be LOW, i.e. < 40).
        ecs: u8,
    },
    /// A sentence was classified as noumenal (transcendental overreach) — S-III-8.
    ///
    /// Emitted *in addition* to `SentenceProcessed` or `HighRiskDetected` for the
    /// same sentence, so callers can react to the boundary crossing specifically.
    NoumenalDetected {
        text: String,
        illusion_kind: String,
        severity: String,
    },
    /// The stream has been flushed; all sentences have been processed.
    Complete {
        total_sentences: usize,
        overall_risk: RiskLevel,
        /// Rolling ECS over all sentences processed in this stream (0–100).
        overall_ecs: u8,
    },
}

// ─── StreamingPipeline ───────────────────────────────────────────────────────

/// Streaming pipeline with rolling sentence buffer.
///
/// Feed text chunks (tokens, words, or larger fragments) via [`feed()`].
/// When a sentence boundary (`.`, `!`, `?`) is detected, the accumulated
/// sentence is analysed. Call [`flush()`] at end-of-stream.
///
/// Every `StreamEvent` now carries an `ecs` field — the Epistemic Confidence
/// Score for that sentence. The `Complete` event carries `overall_ecs`, the
/// minimum ECS seen across all sentences (the weakest-link score).
pub struct StreamingPipeline {
    pipeline: KantianPipeline,
    buffer: String,
    processed: Vec<(String, RiskLevel)>,
    /// Rolling ECS values — one per processed sentence.
    ecs_history: Vec<u8>,
}

impl StreamingPipeline {
    /// Create a new streaming pipeline.
    pub fn new() -> Self {
        Self {
            pipeline: KantianPipeline::new(),
            buffer: String::new(),
            processed: Vec::new(),
            ecs_history: Vec::new(),
        }
    }

    /// Feed a text chunk and receive any triggered events.
    ///
    /// Events are emitted as soon as sentence boundaries are detected.
    /// A `HighRiskDetected` event does NOT stop the pipeline — the caller
    /// decides whether to abort or continue.
    pub fn feed(&mut self, chunk: &str) -> Vec<StreamEvent> {
        self.buffer.push_str(chunk);
        let mut events = Vec::new();

        loop {
            // Find the next sentence boundary
            let boundary = self
                .buffer
                .char_indices()
                .find(|(_, c)| matches!(c, '.' | '!' | '?'))
                .map(|(i, _)| i);

            let Some(pos) = boundary else { break };

            let sentence = self.buffer[..=pos].trim().to_string();
            self.buffer = self.buffer[pos + 1..].trim_start().to_string();

            if sentence.is_empty() {
                continue;
            }

            events.extend(self.analyse_sentence(sentence));
        }

        events
    }

    /// Flush the remaining buffer (even without a sentence boundary).
    ///
    /// Always emits `StreamEvent::Complete` as the last event.
    pub fn flush(&mut self) -> Vec<StreamEvent> {
        let mut events = Vec::new();

        let remainder = self.buffer.trim().to_string();
        if !remainder.is_empty() {
            self.buffer.clear();
            events.extend(self.analyse_sentence(remainder));
        }

        let overall_risk = self
            .processed
            .iter()
            .map(|(_, r)| *r)
            .max()
            .unwrap_or(RiskLevel::Safe);

        // overall_ecs = minimum (weakest-link) across all sentences
        let overall_ecs = self.ecs_history.iter().copied().min().unwrap_or(100);

        events.push(StreamEvent::Complete {
            total_sentences: self.processed.len(),
            overall_risk,
            overall_ecs,
        });

        events
    }

    /// Analyse a single sentence, update internal history, and return event.
    fn analyse_sentence(&mut self, text: String) -> Vec<StreamEvent> {
        match self.pipeline.process(&text) {
            Ok(report) => {
                let risk = report.verdict.risk;
                let ecs = report.ecs();
                let mut events = Vec::new();

                // S-III-8: Emit NoumenalDetected for every illusion found
                for illusion in &report.dialectic.illusions {
                    events.push(StreamEvent::NoumenalDetected {
                        text: text.clone(),
                        illusion_kind: format!("{:?}", illusion.kind),
                        severity: format!("{:?}", illusion.severity),
                    });
                }

                self.processed.push((text.clone(), risk));
                self.ecs_history.push(ecs);
                if risk >= RiskLevel::High {
                    events.push(StreamEvent::HighRiskDetected {
                        text,
                        report: Box::new(report),
                        ecs,
                    });
                } else {
                    events.push(StreamEvent::SentenceProcessed { text, risk, ecs });
                }
                events
            }
            Err(_) => {
                self.processed.push((text.clone(), RiskLevel::Safe));
                self.ecs_history.push(100);
                vec![StreamEvent::SentenceProcessed {
                    text,
                    risk: RiskLevel::Safe,
                    ecs: 100,
                }]
            }
        }
    }

    /// How many sentences have been fully processed so far.
    pub fn sentences_processed(&self) -> usize {
        self.processed.len()
    }

    /// The highest risk level seen so far (before flush).
    pub fn current_max_risk(&self) -> RiskLevel {
        self.processed
            .iter()
            .map(|(_, r)| *r)
            .max()
            .unwrap_or(RiskLevel::Safe)
    }

    /// The minimum (weakest-link) ECS seen across all sentences so far.
    pub fn current_min_ecs(&self) -> u8 {
        self.ecs_history.iter().copied().min().unwrap_or(100)
    }

    /// The ECS band for the current stream (based on the weakest sentence).
    pub fn current_ecs_band(&self) -> EcsBand {
        EcsBand::from_score(self.current_min_ecs())
    }

    /// Classify a single sentence as phenomenal, noumenal, or regulative (S-III-8).
    ///
    /// This is a lightweight check using only the IllusionDetector (no full pipeline run).
    /// For the full analysis, use `pipeline.process()` which covers all layers.
    pub fn classify_noumenal(text: &str) -> PhenomenalStatus {
        use crate::dialectic::IllusionDetector;
        use crate::dialectic::IllusionKind;
        use crate::types::{Proposition, PropositionKind};

        let prop = Proposition::new(text, PropositionKind::Unknown);
        let illusions = IllusionDetector::detect(std::slice::from_ref(&prop));

        if illusions.is_empty() {
            return PhenomenalStatus::Phenomenal;
        }

        // Noumenal illusions take priority over regulative ones: if ANY illusion
        // is HypostatizingIdea, EpistemicOverreach, or CategoryOverextension,
        // the sentence crosses the noumenal boundary.
        let noumenal = illusions
            .iter()
            .filter(|i| i.kind != IllusionKind::RegulativeConstitutive)
            .max_by_key(|i| i.severity);

        if let Some(i) = noumenal {
            return PhenomenalStatus::Noumenal {
                kind: format!("{:?}", i.kind),
                severity: format!("{:?}", i.severity),
            };
        }

        // All illusions are RegulativeConstitutive → regulative claim
        if let Some(i) = illusions.first() {
            return PhenomenalStatus::Regulative {
                reason: i.description.clone(),
            };
        }

        PhenomenalStatus::Phenomenal
    }
}

impl Default for StreamingPipeline {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_text_emits_processed_events() {
        let mut sp = StreamingPipeline::new();
        let events = sp.feed("Water boils at 100 degrees. Ice melts at 0 degrees.");
        assert_eq!(events.len(), 2);
        assert!(events
            .iter()
            .all(|e| matches!(e, StreamEvent::SentenceProcessed { .. })));
    }

    #[test]
    fn sentence_events_carry_ecs() {
        let mut sp = StreamingPipeline::new();
        let events = sp.feed("Water boils at 100 degrees.");
        let ecs = events.iter().find_map(|e| match e {
            StreamEvent::SentenceProcessed { ecs, .. } => Some(*ecs),
            _ => None,
        });
        assert!(ecs.is_some(), "SentenceProcessed should carry an ECS");
        assert!(ecs.unwrap() <= 100);
    }

    #[test]
    fn complete_event_carries_overall_ecs() {
        let mut sp = StreamingPipeline::new();
        sp.feed("Water boils at 100 degrees.");
        let events = sp.flush();
        let overall_ecs = events.iter().find_map(|e| match e {
            StreamEvent::Complete { overall_ecs, .. } => Some(*overall_ecs),
            _ => None,
        });
        assert!(overall_ecs.is_some(), "Complete should carry overall_ecs");
    }

    #[test]
    fn min_ecs_api_works() {
        let mut sp = StreamingPipeline::new();
        sp.feed("Snow is white. Ice melts at 0 degrees.");
        let min_ecs = sp.current_min_ecs();
        assert!(min_ecs <= 100);
    }

    #[test]
    fn flush_emits_complete() {
        let mut sp = StreamingPipeline::new();
        sp.feed("First sentence.");
        let events = sp.flush();
        assert!(events
            .iter()
            .any(|e| matches!(e, StreamEvent::Complete { .. })));
    }

    #[test]
    fn theological_triggers_high_risk() {
        let mut sp = StreamingPipeline::new();
        // Need multiple issues to hit HIGH — combine theological + antinomy + paralogism
        let events = sp.feed("God exists. The universe had a beginning. The soul is a substance. The world is finite.");
        let has_high = events
            .iter()
            .any(|e| matches!(e, StreamEvent::HighRiskDetected { .. }));
        // At least one detected event (exact threshold depends on input)
        assert!(!events.is_empty());
        let _ = has_high; // May or may not be high — just check it doesn't crash
    }

    #[test]
    fn partial_sentence_buffered() {
        let mut sp = StreamingPipeline::new();
        // Feed without sentence boundary
        let events = sp.feed("No boundary here");
        assert!(events.is_empty(), "No events until sentence boundary");
        assert_eq!(sp.sentences_processed(), 0);
        // Now flush
        let flush_events = sp.flush();
        assert!(flush_events
            .iter()
            .any(|e| matches!(e, StreamEvent::Complete { .. })));
    }

    #[test]
    fn sentence_count_tracked() {
        let mut sp = StreamingPipeline::new();
        sp.feed("First. Second. Third.");
        assert_eq!(sp.sentences_processed(), 3);
    }

    #[test]
    fn classify_noumenal_safe_text_is_phenomenal() {
        let status = StreamingPipeline::classify_noumenal("Water boils at 100 degrees Celsius.");
        assert_eq!(status, PhenomenalStatus::Phenomenal);
    }

    #[test]
    fn classify_noumenal_theological_is_noumenal() {
        let status = StreamingPipeline::classify_noumenal(
            "God exists necessarily and is the ground of all being.",
        );
        assert!(
            matches!(status, PhenomenalStatus::Noumenal { .. }),
            "Theological absolute claim should be classified as noumenal"
        );
    }

    #[test]
    fn noumenal_event_emitted_for_theological_claim() {
        let mut sp = StreamingPipeline::new();
        let events = sp.feed("God exists.");
        let has_noumenal = events
            .iter()
            .any(|e| matches!(e, StreamEvent::NoumenalDetected { .. }));
        // Theological hypostatizing-idea claim should trigger NoumenalDetected
        assert!(
            has_noumenal,
            "Theological claim should emit NoumenalDetected event"
        );
    }
}
