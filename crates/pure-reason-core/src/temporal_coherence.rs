//! # Temporal Coherence Layer (TRIZ Report VIII — S4)
//!
//! Detects temporal contradictions and recency overreach within a document.
//!
//! ## Mechanisms
//!
//! 1. **Cross-sentence date contradiction**: two sentences reference the same
//!    named entity but assert different years/dates → contradiction flagged.
//!
//! 2. **Recency overreach**: the text uses "recently", "just", "new", "latest"
//!    adjacent to a year that is more than 5 years in the past → overreach flagged.
//!
//! 3. **Anachronism detection**: an entity is placed in a time period inconsistent
//!    with its well-known existence dates (e.g., Einstein inventing the internet).
//!
//! ## Design (TRIZ IFR)
//!
//! All checks are single-pass, zero-heap on the clean path.
//! Year extraction uses ASCII digit scanning without regex.
//! The current year is determined at compile time via a static heuristic
//! (2025 — can be updated as needed, or overridden in tests).

use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};

// ─── Current year constant ────────────────────────────────────────────────────

/// The current year used for recency overreach checks.
/// Derived dynamically so the detector does not go stale every January.
fn current_year() -> u32 {
    Utc::now().year().max(0) as u32
}

/// Maximum number of years in the past that counts as "recent".
const RECENCY_THRESHOLD_YEARS: u32 = 5;

// ─── Issue types ──────────────────────────────────────────────────────────────

/// A detected temporal coherence issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalIssue {
    pub kind: TemporalIssueKind,
    pub description: String,
    /// The sentence(s) involved (0-indexed).
    pub sentence_indices: Vec<usize>,
}

/// What kind of temporal coherence problem was detected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TemporalIssueKind {
    /// Two sentences assert different years for the same subject.
    CrossSentenceContradiction,
    /// "Recently" / "just" used with a year older than RECENCY_THRESHOLD_YEARS.
    RecencyOverreach,
    /// An entity is placed in a time period inconsistent with known history.
    Anachronism,
}

impl std::fmt::Display for TemporalIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[TCL:{:?}] {}", self.kind, self.description)
    }
}

// ─── Scanner ─────────────────────────────────────────────────────────────────

/// Temporal coherence scanner.
///
/// Call `scan(sentences)` with the split sentences of the answer/document.
pub struct TemporalCoherenceLayer;

impl TemporalCoherenceLayer {
    /// Scan a list of sentences for temporal coherence issues.
    ///
    /// Returns a (possibly empty) list of detected issues.
    pub fn scan(sentences: &[&str]) -> Vec<TemporalIssue> {
        let mut issues = Vec::new();

        // 1. Extract (year, sentence_index) pairs for each sentence.
        let year_map: Vec<Vec<u32>> = sentences.iter().map(|s| extract_years(s)).collect();

        // 2. Cross-sentence contradiction check.
        // Heuristic: if two adjacent sentences reference the same subject noun
        // and different years, flag it.
        issues.extend(Self::check_cross_sentence_contradiction(
            sentences, &year_map,
        ));

        // 3. Recency overreach check.
        issues.extend(Self::check_recency_overreach(sentences, &year_map));

        // 4. Anachronism check.
        issues.extend(Self::check_anachronisms(sentences));

        issues
    }

    fn check_cross_sentence_contradiction(
        sentences: &[&str],
        year_map: &[Vec<u32>],
    ) -> Vec<TemporalIssue> {
        let mut issues = Vec::new();
        if sentences.len() < 2 {
            return issues;
        }

        // Compare each pair of adjacent sentences (window = 3 for broader scope).
        for window_start in 0..sentences.len().saturating_sub(1) {
            let window_end = (window_start + 3).min(sentences.len());
            for i in window_start..window_end {
                for j in (i + 1)..window_end {
                    let years_i = &year_map[i];
                    let years_j = &year_map[j];
                    if years_i.is_empty() || years_j.is_empty() {
                        continue;
                    }
                    // Check if subjects overlap
                    if !Self::sentences_share_subject(sentences[i], sentences[j]) {
                        continue;
                    }
                    // Check if any year in i contradicts any year in j (non-overlapping)
                    for &yi in years_i {
                        for &yj in years_j {
                            let diff = yi.max(yj) - yi.min(yj);
                            // Flag if same subject has wildly different years (>20 year gap is suspicious)
                            if diff > 20 && diff < 2000 {
                                issues.push(TemporalIssue {
                                    kind: TemporalIssueKind::CrossSentenceContradiction,
                                    description: format!(
                                        "Sentences {} and {} reference the same subject with contradictory years ({} vs {}; gap: {} years)",
                                        i, j, yi, yj, diff
                                    ),
                                    sentence_indices: vec![i, j],
                                });
                            }
                        }
                    }
                }
            }
        }
        issues
    }

    /// Rough subject overlap: check if first 3 non-stop-word tokens overlap.
    fn sentences_share_subject(a: &str, b: &str) -> bool {
        let keywords_a = content_keywords(a);
        let keywords_b = content_keywords(b);
        keywords_a.iter().any(|w| keywords_b.contains(w))
    }

    fn check_recency_overreach(sentences: &[&str], year_map: &[Vec<u32>]) -> Vec<TemporalIssue> {
        let mut issues = Vec::new();
        let recency_phrases = [
            "recently",
            "just ",
            "newly",
            "latest ",
            "new study",
            "new research",
            "just announced",
            "just released",
            "just published",
            "brand new",
            "cutting-edge",
            "state-of-the-art",
            "most recent",
            "up-to-date",
        ];

        for (i, sentence) in sentences.iter().enumerate() {
            let lower = sentence.to_lowercase();
            let has_recency = recency_phrases.iter().any(|p| lower.contains(p));
            if !has_recency {
                continue;
            }
            for &year in &year_map[i] {
                let current_year = current_year();
                if year > 1000 && year <= current_year {
                    let age = current_year - year;
                    if age > RECENCY_THRESHOLD_YEARS {
                        issues.push(TemporalIssue {
                            kind: TemporalIssueKind::RecencyOverreach,
                            description: format!(
                                "Sentence {} uses recency language ('recently'/'new'/etc.) but references year {} ({} years ago)",
                                i, year, age
                            ),
                            sentence_indices: vec![i],
                        });
                    }
                }
            }
        }
        issues
    }

    fn check_anachronisms(sentences: &[&str]) -> Vec<TemporalIssue> {
        let mut issues = Vec::new();

        // Static anachronism patterns: (entity_signal, anachronistic_context_signals)
        // Each entry: if entity_signal and any context_signal co-occur in same sentence → flag.
        static ANACHRONISM_PATTERNS: &[(&str, &[&str], &str)] = &[
            (
                "newton",
                &["computer", "electricity", "internet", "quantum", "nuclear", "television", "radio"],
                "Isaac Newton (1643–1727) could not have interacted with post-18th-century technology",
            ),
            (
                "einstein",
                &["internet", "smartphone", "artificial intelligence", "ai model", "chatgpt", "social media"],
                "Einstein (1879–1955) predates the internet and modern AI",
            ),
            (
                "shakespeare",
                &["telephone", "electricity", "internet", "computer", "photograph", "film"],
                "Shakespeare (1564–1616) predates all electrical/digital technology",
            ),
            (
                "da vinci",
                &["internet", "electricity", "computer", "nuclear", "telephone"],
                "Leonardo da Vinci (1452–1519) predates electricity and modern technology",
            ),
            (
                "cleopatra",
                &["airplane", "car", "electricity", "internet", "computer", "telephone", "nuclear"],
                "Cleopatra (69–30 BC) predates all modern technology by two millennia",
            ),
            (
                "julius caesar",
                &["gun", "gunpowder", "airplane", "car", "internet", "electricity"],
                "Julius Caesar (100–44 BC) predates firearms and modern technology",
            ),
            (
                "dinosaur",
                &["human", "humans", "caveman", "homo sapiens", "modern human"],
                "Non-avian dinosaurs went extinct ~66 million years before modern humans evolved",
            ),
            (
                "medieval",
                &["electricity", "computer", "internet", "gun", "automobile"],
                "Medieval period (5th–15th century) predates electricity and motor vehicles",
            ),
        ];

        for (i, sentence) in sentences.iter().enumerate() {
            let lower = sentence.to_lowercase();
            for (entity_signal, context_signals, explanation) in ANACHRONISM_PATTERNS {
                if !lower.contains(entity_signal) {
                    continue;
                }
                for ctx in *context_signals {
                    if lower.contains(ctx) {
                        issues.push(TemporalIssue {
                            kind: TemporalIssueKind::Anachronism,
                            description: format!(
                                "Sentence {}: potential anachronism — {} co-occurs with '{}'. {}",
                                i, entity_signal, ctx, explanation
                            ),
                            sentence_indices: vec![i],
                        });
                        break; // One flag per entity per sentence
                    }
                }
            }
        }
        issues
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Extract all 4-digit year-like numbers (1000–2099) from text.
fn extract_years(text: &str) -> Vec<u32> {
    let mut years = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 4 <= bytes.len() {
        if bytes[i].is_ascii_digit()
            && bytes[i + 1].is_ascii_digit()
            && bytes[i + 2].is_ascii_digit()
            && bytes[i + 3].is_ascii_digit()
            && (i == 0 || !bytes[i - 1].is_ascii_digit())
            && (i + 4 >= bytes.len() || !bytes[i + 4].is_ascii_digit())
        {
            let year: u32 = (bytes[i] - b'0') as u32 * 1000
                + (bytes[i + 1] - b'0') as u32 * 100
                + (bytes[i + 2] - b'0') as u32 * 10
                + (bytes[i + 3] - b'0') as u32;
            if (1000..=2099).contains(&year) {
                years.push(year);
            }
            i += 4;
        } else {
            i += 1;
        }
    }
    years
}

/// Extract meaningful content keywords (skip stop words).
fn content_keywords(text: &str) -> Vec<String> {
    static STOP_WORDS: &[&str] = &[
        "the", "a", "an", "is", "was", "were", "are", "be", "been", "in", "on", "at", "to", "of",
        "and", "or", "but", "with", "that", "this", "it", "he", "she", "they", "we", "i", "have",
        "has", "had", "will", "would", "could", "should", "not", "no", "by", "from", "for", "as",
        "into", "about",
    ];

    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 4)
        .map(|w| w.to_lowercase())
        .filter(|w| !STOP_WORDS.contains(&w.as_str()))
        .collect()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recency_overreach_fires() {
        let sentences = vec!["This is a recently published study from 2015 on climate change."];
        let issues = TemporalCoherenceLayer::scan(&sentences);
        assert!(
            !issues.is_empty(),
            "Should detect recency overreach for 2015"
        );
        assert_eq!(issues[0].kind, TemporalIssueKind::RecencyOverreach);
    }

    #[test]
    fn recency_overreach_current_year_passes() {
        let sentences = vec!["This is a recently published study from 2024 on climate change."];
        let issues = TemporalCoherenceLayer::scan(&sentences);
        let overreach: Vec<_> = issues
            .iter()
            .filter(|i| i.kind == TemporalIssueKind::RecencyOverreach)
            .collect();
        assert!(
            overreach.is_empty(),
            "2024 is recent enough: {:?}",
            overreach
        );
    }

    #[test]
    fn anachronism_einstein_internet_fires() {
        let sentences = vec!["Einstein used the internet to publish his relativity paper."];
        let issues = TemporalCoherenceLayer::scan(&sentences);
        assert!(
            !issues.is_empty(),
            "Einstein + internet should be anachronism"
        );
        assert_eq!(issues[0].kind, TemporalIssueKind::Anachronism);
    }

    #[test]
    fn clean_text_no_issues() {
        let sentences = vec![
            "The French Revolution began in 1789.",
            "It fundamentally transformed France.",
        ];
        let issues = TemporalCoherenceLayer::scan(&sentences);
        assert!(
            issues.is_empty(),
            "Clean text should produce no issues: {:?}",
            issues
        );
    }

    #[test]
    fn extract_years_basic() {
        let years = extract_years("In 1789 the revolution started; by 1799 Napoleon took power.");
        assert_eq!(years, vec![1789, 1799]);
    }

    #[test]
    fn extract_years_ignores_non_year_digits() {
        let years = extract_years("12345 is not a year; 2023 is.");
        assert_eq!(years, vec![2023]);
    }
}
