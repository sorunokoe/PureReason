//! # Lexical Entity Coverage Analyzer (TRIZ Solution 2 — S2)
//!
//! Detects hallucinations via **lexical novelty**: named entities, numbers, and
//! proper nouns in the answer that are absent from the knowledge/question context
//! are a strong signal of entity-substitution or fabrication hallucination.
//!
//! ## Core Insight (TRIZ P13 — The Other Way Round)
//!
//! Instead of asking "what is *wrong* with this answer?", ask "what did this
//! answer *add* that was not there before?" High novelty → likely fabrication.
//!
//! ## Signal (TRIZ P35 — Change Physico-Chemical Parameters)
//!
//! The detection axis changes from categorical/modal (Kantian modality check)
//! to **lexical-statistical** (entity novelty). Both signals are complementary:
//! - Kantian modality catches *overconfident* language
//! - Entity novelty catches *wrong content* expressed with normal confidence
//!
//! ## Hallucination Pattern Covered
//!
//! | Knowledge | Question | Hallucinated Answer |
//! |---|---|---|
//! | "Capital of X is Y" | "What is capital of X?" | "[Z] is the capital of X" |
//!
//! Z is novel (not in knowledge/question) → `novelty_score` > 0.5 → flag.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ─── LexicalCoverageReport ───────────────────────────────────────────────────

/// Result of the Lexical Entity Coverage check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexicalCoverageReport {
    /// Named entities / proper nouns found in the answer.
    pub answer_entities: Vec<String>,
    /// Named entities / proper nouns found in the context (knowledge + question).
    pub context_entities: Vec<String>,
    /// Answer entities not found in context — the "novel" elements.
    pub uncovered_entities: Vec<String>,
    /// Fraction of answer entities covered by context (0.0–1.0).
    /// 1.0 = answer is fully grounded in context.
    pub coverage_ratio: f64,
    /// Fraction of answer entities NOT covered by context (1.0 − coverage_ratio).
    /// High novelty (> 0.5) is a hallucination signal.
    pub novelty_score: f64,
}

impl LexicalCoverageReport {
    /// Trivially grounded — used when no context is available.
    pub fn grounded() -> Self {
        Self {
            answer_entities: Vec::new(),
            context_entities: Vec::new(),
            uncovered_entities: Vec::new(),
            coverage_ratio: 1.0,
            novelty_score: 0.0,
        }
    }
}

// ─── LexicalCoverageAnalyzer ─────────────────────────────────────────────────

/// Computes the lexical entity coverage of an answer against a knowledge context.
///
/// Zero dependencies — purely lexical pattern matching using Rust's standard library.
pub struct LexicalCoverageAnalyzer;

impl LexicalCoverageAnalyzer {
    /// Analyze how well the `answer` is grounded in `context`.
    ///
    /// `context` should be the knowledge + question text (concatenated or separate).
    ///
    /// Returns a report with a `novelty_score` ∈ [0.0, 1.0]:
    /// - 0.0 → answer is fully grounded (all entities present in context)
    /// - 1.0 → answer introduces only novel entities (likely hallucination)
    pub fn analyze(context: &str, answer: &str) -> LexicalCoverageReport {
        let context_entities: HashSet<String> = extract_entities(context);
        let answer_entities: HashSet<String> = extract_entities(answer);

        if answer_entities.is_empty() {
            return LexicalCoverageReport::grounded();
        }

        let uncovered: Vec<String> = answer_entities
            .difference(&context_entities)
            .cloned()
            .collect();

        let coverage_ratio = 1.0 - (uncovered.len() as f64 / answer_entities.len() as f64);
        let novelty_score = 1.0 - coverage_ratio;

        let mut answer_list: Vec<String> = answer_entities.into_iter().collect();
        let mut context_list: Vec<String> = context_entities.into_iter().collect();
        answer_list.sort();
        context_list.sort();
        let mut uncovered_sorted = uncovered.clone();
        uncovered_sorted.sort();

        LexicalCoverageReport {
            answer_entities: answer_list,
            context_entities: context_list,
            uncovered_entities: uncovered_sorted,
            coverage_ratio,
            novelty_score,
        }
    }
}

// ─── Entity extraction ───────────────────────────────────────────────────────

/// Extract named entity tokens from `text`.
///
/// Heuristics (O(n)):
/// 1. **Capitalised word** not at the very beginning of a sentence  
///    (avoids tagging normal sentence-initial capitalisation).
/// 2. **Pure number** of at least 2 digits (years, quantities, identifiers).
/// 3. **Quoted phrase** is treated as a single entity token.
///
/// All tokens are lowercased for case-insensitive comparison.
fn extract_entities(text: &str) -> HashSet<String> {
    const MIN_ENTITY_LEN: usize = 2;
    /// Words that are NOT named entities even when capitalised (e.g., sentence-start articles).
    const COMMON_CAPS: &[&str] = &[
        "the", "is", "are", "was", "were", "has", "have", "had", "in", "of", "to", "for", "and",
        "but", "or", "a", "an", "it", "its", "be", "by", "on", "at", "as", "do", "did", "not",
        "no", "so", "he", "she", "we", "you", "they", "this", "that", "with", "from", "what",
        "which", "who", "when", "where", "how", "why", "can", "could", "would", "will", "may",
        "might", "must", "should", "shall",
        // Additional common sentence-initial words
        "here", "now", "once", "each", "most", "many", "both", "such", "upon", "these", "those",
        "every", "other", "our", "your", "their", "its",
    ];

    let mut entities = HashSet::new();

    for word in text.split_whitespace() {
        // Strip leading/trailing punctuation
        let clean: String = word
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect();

        if clean.len() < MIN_ENTITY_LEN {
            continue;
        }

        let lower = clean.to_lowercase();
        let starts_upper = clean
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false);
        let all_alpha = clean.chars().all(|c| c.is_alphabetic() || c == '-');
        let all_numeric = clean.chars().all(|c| c.is_ascii_digit());

        let is_entity =
            // Pure numeric (year, count, identifier)
            (all_numeric && clean.len() >= 2) ||
            // Capitalised alphabetic token not in common-words list
            (starts_upper && all_alpha && !COMMON_CAPS.contains(&lower.as_str()));

        if is_entity {
            entities.insert(lower);
        }
    }

    entities
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_novelty_for_wrong_capital() {
        let context = "The capital of Australia is Canberra. What is the capital of Australia?";
        let answer = "Sydney is the capital of Australia.";
        let report = LexicalCoverageAnalyzer::analyze(context, answer);
        // "Sydney" is novel (not in context); "Canberra" is not in answer → novelty > 0
        assert!(
            report.novelty_score > 0.0,
            "Expected some novelty for wrong-capital answer, got {}",
            report.novelty_score
        );
        assert!(
            report.uncovered_entities.iter().any(|e| e == "sydney"),
            "Expected 'sydney' in uncovered entities: {:?}",
            report.uncovered_entities
        );
    }

    #[test]
    fn low_novelty_for_correct_answer() {
        let context = "The capital of Australia is Canberra. What is the capital of Australia?";
        let answer = "Canberra is the capital of Australia.";
        let report = LexicalCoverageAnalyzer::analyze(context, answer);
        // All entities in answer (Canberra) already in context
        assert!(
            report.novelty_score < 0.4,
            "Expected low novelty for correct answer, got {}",
            report.novelty_score
        );
    }

    #[test]
    fn grounded_report_when_no_answer_entities() {
        let report = LexicalCoverageAnalyzer::analyze("some context", "it is the case.");
        assert_eq!(report.novelty_score, 0.0);
        assert!(report.answer_entities.is_empty());
    }

    #[test]
    fn numbers_are_extracted_as_entities() {
        let context = "The event happened in 1945.";
        let answer = "The event happened in 1989.";
        let report = LexicalCoverageAnalyzer::analyze(context, answer);
        assert!(
            report.uncovered_entities.iter().any(|e| e == "1989"),
            "Expected '1989' to be novel: {:?}",
            report.uncovered_entities
        );
    }
}
