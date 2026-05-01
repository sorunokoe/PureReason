//! # Input Segmenter (TRIZ Solution 1 — S1)
//!
//! Parses structured benchmark-style inputs into named segments.
//!
//! Many hallucination benchmarks (HaluEval QA, HaluEval Dialogue, TruthfulQA)
//! embed a knowledge oracle directly inside the input text:
//!
//! ```text
//! Knowledge: The capital of Australia is Canberra.
//! Question:  What is the capital of Australia?
//! Answer:    Sydney is the capital of Australia.
//! ```
//!
//! The pipeline previously treated this as one undifferentiated text blob —
//! discarding the knowledge-answer relationship that reveals the hallucination.
//!
//! `InputSegmenter` splits on standard benchmark headers and exposes each
//! segment as a separate string for targeted cross-segment analysis.
//!
//! **TRIZ Principles applied:**
//! - P1 (Segmentation): divide the input into independent, role-tagged parts
//! - P22 (Turn Harm into Benefit): the "Knowledge:" prefix was noise; now it is the oracle
//! - P25 (Self-Service): the verification oracle is already inside the system's input

use serde::{Deserialize, Serialize};

// ─── SegmentedInput ──────────────────────────────────────────────────────────

/// A structured input parsed into named segments.
///
/// Handles the common benchmark format:
/// `"Knowledge: {k}\nQuestion: {q}\nAnswer: {a}"`
/// and the dialogue variant:
/// `"Knowledge: {k}\nResponse: {r}"`
/// as well as bare `"Question: {q}\nAnswer: {a}"` without knowledge.
///
/// When none of the structural headers are found, `raw` is the only populated
/// field and the other fields are `None` — matching the behaviour of an
/// unstructured input.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SegmentedInput {
    /// The full original text, unchanged.
    pub raw: String,

    /// The "Knowledge: …" segment, if present.
    /// Contains the factual context against which the answer should be checked.
    pub knowledge: Option<String>,

    /// The "Question: …" segment, if present.
    pub question: Option<String>,

    /// The "Answer: …" or "Response: …" segment, if present.
    /// This is the claim being evaluated for hallucination.
    pub answer: Option<String>,
}

impl SegmentedInput {
    /// Parse raw text into a `SegmentedInput`.
    ///
    /// Detection is O(n) — a single pass over the text looking for line-level headers.
    /// Headers are matched case-insensitively and may appear in any order.
    pub fn parse(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let mut knowledge: Option<String> = None;
        let mut question: Option<String> = None;
        let mut answer: Option<String> = None;

        // Split into lines and group by header
        let mut current_key: Option<&'static str> = None;
        let mut current_val = String::new();

        let flush = |key: Option<&'static str>,
                     val: &str,
                     k: &mut Option<String>,
                     q: &mut Option<String>,
                     a: &mut Option<String>| {
            let trimmed = val.trim().to_string();
            if trimmed.is_empty() {
                return;
            }
            match key {
                Some("knowledge") => *k = Some(trimmed),
                Some("question") => *q = Some(trimmed),
                Some("answer") => *a = Some(trimmed),
                _ => {}
            }
        };

        for line in raw.lines() {
            if let Some(seg_key) = detect_header(line) {
                // Flush the previous segment
                flush(
                    current_key,
                    &current_val,
                    &mut knowledge,
                    &mut question,
                    &mut answer,
                );
                current_key = Some(seg_key);
                current_val = after_header(line).to_string();
            } else {
                // Continuation of the current segment
                if current_key.is_some() {
                    if !current_val.is_empty() {
                        current_val.push(' ');
                    }
                    current_val.push_str(line.trim());
                }
            }
        }
        // Flush the last segment
        flush(
            current_key,
            &current_val,
            &mut knowledge,
            &mut question,
            &mut answer,
        );

        Self {
            raw,
            knowledge,
            question,
            answer,
        }
    }

    /// Returns `true` when both a knowledge context and an answer are present.
    ///
    /// Only when this is true should the Knowledge-Answer Contradiction (KAC)
    /// engine be activated — running it without a knowledge context would yield
    /// false positives by comparing unrelated text segments.
    pub fn has_knowledge_answer_context(&self) -> bool {
        self.knowledge.is_some() && self.answer.is_some()
    }

    /// Returns `true` when only a question and answer are present (no knowledge).
    ///
    /// In this mode a weaker lexical consistency check is applied: the answer
    /// should not introduce named entities absent from the question.
    pub fn has_question_answer_context(&self) -> bool {
        self.question.is_some() && self.answer.is_some() && self.knowledge.is_none()
    }

    /// Returns `true` when the input is a dialogue-format pair (Knowledge + Response,
    /// no Question header). This suppresses FP-prone detectors in `compose_verdict`.
    ///
    /// Dialogue responses use hedged, first-person language that triggers illusion/
    /// paralogism detectors even when the answer is correct. For dialogue, only the
    /// knowledge-contradiction check is reliable.
    pub fn is_dialogue_format(&self) -> bool {
        self.knowledge.is_some() && self.answer.is_some() && self.question.is_none()
    }

    /// The best available "context" string against which to validate the answer.
    ///
    /// Prefers the knowledge segment; falls back to the question segment.
    pub fn context(&self) -> Option<&str> {
        self.knowledge.as_deref().or(self.question.as_deref())
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Detect whether a line starts with a known segment header.
/// Returns a canonical lowercase key or `None`.
fn detect_header(line: &str) -> Option<&'static str> {
    let lower = line.to_lowercase();
    let lower = lower.trim_start();
    if lower.starts_with("knowledge:") {
        return Some("knowledge");
    }
    if lower.starts_with("question:") {
        return Some("question");
    }
    if lower.starts_with("answer:") {
        return Some("answer");
    }
    if lower.starts_with("response:") {
        return Some("answer");
    }
    None
}

/// Return the text after the header prefix (e.g., after "Knowledge: ").
fn after_header(line: &str) -> &str {
    let colon = line.find(':').unwrap_or(0);
    line[colon + 1..].trim()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_halueval_qa_format() {
        let text = "Knowledge: The capital of Australia is Canberra.\n\
                    Question: What is the capital of Australia?\n\
                    Answer: Sydney is the capital of Australia.";
        let seg = SegmentedInput::parse(text);
        assert_eq!(
            seg.knowledge.as_deref(),
            Some("The capital of Australia is Canberra.")
        );
        assert_eq!(
            seg.question.as_deref(),
            Some("What is the capital of Australia?")
        );
        assert_eq!(
            seg.answer.as_deref(),
            Some("Sydney is the capital of Australia.")
        );
        assert!(seg.has_knowledge_answer_context());
    }

    #[test]
    fn parses_dialogue_format_with_response_key() {
        let text = "Knowledge: Paris is the capital of France.\n\
                    Response: The capital of France is Lyon.";
        let seg = SegmentedInput::parse(text);
        assert!(seg.knowledge.is_some());
        assert!(seg.answer.is_some());
        assert!(seg.has_knowledge_answer_context());
        // No Question: header → dialogue format
        assert!(seg.is_dialogue_format());
    }

    #[test]
    fn qa_with_question_is_not_dialogue_format() {
        let text = "Knowledge: Paris is the capital of France.\n\
                    Question: What is the capital of France?\n\
                    Answer: Lyon is the capital of France.";
        let seg = SegmentedInput::parse(text);
        assert!(seg.has_knowledge_answer_context());
        // Has Question: header → not dialogue
        assert!(!seg.is_dialogue_format());
    }

    #[test]
    fn unstructured_input_is_unchanged() {
        let text = "Every event must have a cause.";
        let seg = SegmentedInput::parse(text);
        assert_eq!(seg.raw, text);
        assert!(seg.knowledge.is_none());
        assert!(seg.answer.is_none());
        assert!(!seg.has_knowledge_answer_context());
    }

    #[test]
    fn question_answer_without_knowledge() {
        let text = "Question: Who wrote Hamlet?\nAnswer: Shakespeare wrote Hamlet.";
        let seg = SegmentedInput::parse(text);
        assert!(seg.has_question_answer_context());
        assert!(!seg.has_knowledge_answer_context());
    }
}
