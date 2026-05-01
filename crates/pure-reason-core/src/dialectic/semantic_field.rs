//! # SemanticField Trait (TRIZ P1 / Solution A-1)
//!
//! Abstracts the similarity backend used by the Dialectic layer for contradiction
//! and antinomy detection. This resolves **TC-7** (semantic depth vs zero-dependency):
//!
//! - `KeywordSemanticField` — the default, zero-dependency implementation using
//!   Jaccard word-overlap and negation polarity. Same accuracy as v0.1.0.
//! - Any richer embedding-based implementation can still be added in a downstream
//!   crate without touching `pure-reason-core`.
//!
//! The trait is object-safe: all methods take `&str` and return `f64`, so it
//! can be used as `Box<dyn SemanticField>` or `Arc<dyn SemanticField>`.

// ─── SemanticField Trait ─────────────────────────────────────────────────────

/// A pluggable semantic similarity backend.
///
/// Implementations range from zero-cost keyword overlap (default) to
/// embedding-model cosine similarity supplied by a downstream crate.
pub trait SemanticField: Send + Sync {
    /// Semantic similarity between two text fragments (0.0 = unrelated, 1.0 = identical).
    fn similarity(&self, a: &str, b: &str) -> f64;

    /// Probability that `counter` is a negation or contradiction of `claim` (0.0–1.0).
    ///
    /// Returns values closer to 1.0 for strong contradictions (e.g. "A is true" vs "A is false").
    fn is_negation_of(&self, claim: &str, counter: &str) -> f64;

    /// Category-level semantic distance using a 12-dimensional category score vector.
    ///
    /// Two propositions are categorically distant if their top category differs
    /// (e.g. Causality vs Necessity — different epistemic modes).
    /// Default implementation returns 0.5 (neutral — override with embedding model).
    fn category_distance(&self, _a_top_category: &str, _b_top_category: &str) -> f64 {
        0.5
    }

    /// Detect entity-substitution hallucination: the answer discusses the same topic
    /// as the knowledge but names different specific entities (wrong city, date, person).
    ///
    /// Returns 0.0 when unrelated texts, consistent entities, or insufficient signal.
    /// Returns up to 1.0 when same topic but clearly different named entities.
    ///
    /// Default implementation returns 0.0 (override for richer entity detection).
    fn entity_conflict_score(&self, _knowledge: &str, _answer: &str) -> f64 {
        0.0
    }
}

// ─── KeywordSemanticField ────────────────────────────────────────────────────

/// Zero-dependency keyword-overlap implementation of `SemanticField`.
///
/// Uses Jaccard similarity on word sets for `similarity()`, and
/// negation-polarity detection for `is_negation_of()`. This is the
/// current v0.1.0 detection logic, now accessible via the trait.
#[derive(Debug, Default, Clone)]
pub struct KeywordSemanticField;

impl SemanticField for KeywordSemanticField {
    fn similarity(&self, a: &str, b: &str) -> f64 {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();
        let a_words: std::collections::HashSet<&str> = a_lower.split_whitespace().collect();
        let b_words: std::collections::HashSet<&str> = b_lower.split_whitespace().collect();

        // Jaccard index: |A ∩ B| / |A ∪ B|
        let intersection = a_words.intersection(&b_words).count();
        let union = a_words.union(&b_words).count();
        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    fn is_negation_of(&self, claim: &str, counter: &str) -> f64 {
        let c1 = claim.to_lowercase();
        let c2 = counter.to_lowercase();

        let negation_words = [
            "not",
            "no",
            "never",
            "false",
            "incorrect",
            "wrong",
            "doesn't",
            "cannot",
            "impossible",
        ];

        // CRITICAL FIX: use whole-word matching, not substring.
        // Substring `contains("no")` fires on "knowledge", "unknown", "annotation", etc.
        // Substring `contains("not")` fires on "another", "cannot", "notation", etc.
        // Word-boundary matching eliminates these catastrophic false positives.
        let has_negation_word = |text: &str| -> bool {
            text.split(|c: char| !c.is_alphanumeric() && c != '\'')
                .any(|tok| negation_words.contains(&tok))
        };

        let c1_negated = has_negation_word(&c1);
        let c2_negated = has_negation_word(&c2);

        // If polarity differs, check word overlap to confirm same topic
        if c1_negated != c2_negated {
            let sim = self.similarity(claim, counter);
            // Strong contradiction: different polarity + shared topic (≥25% word overlap)
            if sim >= 0.25 {
                return 0.6 + (sim * 0.4); // 0.6–1.0 range
            }
        }

        // If both have negation or neither, low contradiction probability
        if c1_negated == c2_negated {
            return 0.05;
        }

        0.0
    }

    /// Override with categorical distance based on keyword category signals.
    fn category_distance(&self, a_top_category: &str, b_top_category: &str) -> f64 {
        if a_top_category == b_top_category {
            return 0.0;
        }
        // Categories within the same group are semantically close
        let quantity = ["Unity", "Plurality", "Totality"];
        let quality = ["Reality", "Negation", "Limitation"];
        let relation = ["Substance", "Causality", "Community"];
        let modality = ["Possibility", "Existence", "Necessity"];
        let groups = [&quantity[..], &quality[..], &relation[..], &modality[..]];

        let group_of = |cat: &str| -> Option<usize> {
            groups.iter().enumerate().find_map(
                |(i, g)| {
                    if g.contains(&cat) {
                        Some(i)
                    } else {
                        None
                    }
                },
            )
        };

        match (group_of(a_top_category), group_of(b_top_category)) {
            (Some(ga), Some(gb)) if ga == gb => 0.2, // same group → close
            (Some(_), Some(_)) => 0.8,               // different group → distant
            _ => 0.5,
        }
    }

    /// Detect entity-substitution hallucination.
    ///
    /// Algorithm (O(n)):
    /// 1. Extract "topic words" (lowercase, stop-word-filtered, len ≥ 4)
    /// 2. Extract "entity tokens" (starts with uppercase — excluding STOP words —
    ///    or 2+ digit pure numbers). Sentence-start words are included if not in STOP.
    /// 3. Topic overlap (Jaccard) ≥ 0.15 → same subject matter
    /// 4. Entity Jaccard distance > 0.35 → different specific entities
    /// 5. Score = topic_overlap × entity_conflict
    fn entity_conflict_score(&self, knowledge: &str, answer: &str) -> f64 {
        const STOP: &[&str] = &[
            "the", "and", "that", "this", "with", "from", "have", "been", "were", "they", "what",
            "when", "which", "then", "than", "also", "into", "over", "after", "some", "more",
            "very", "just", "will", "your", "about", "there", "their", "would", "could", "should",
            // Additional sentence-start common words
            "here", "now", "once", "each", "most", "many", "both", "such", "upon", "these", "those",
            "every", "other", "its", "our",
        ];

        let extract = |text: &str| -> (
            std::collections::HashSet<String>,
            std::collections::HashSet<String>,
        ) {
            let mut entities: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut topics: std::collections::HashSet<String> = std::collections::HashSet::new();
            for raw_word in text.split_whitespace() {
                let word: String = raw_word
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '-')
                    .collect();
                if word.len() < 2 {
                    continue;
                }

                let starts_upper = word
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);
                let all_alpha = word.chars().all(|c| c.is_alphabetic() || c == '-');
                let all_numeric = word.chars().all(|c| c.is_numeric());
                let lower = word.to_lowercase();

                // Entity: number ≥ 2 digits, OR capitalized word not in STOP list
                if (all_numeric && word.len() >= 2)
                    || (starts_upper && all_alpha && !STOP.contains(&lower.as_str()))
                {
                    entities.insert(lower.clone());
                }

                if all_alpha && word.len() >= 4 && !STOP.contains(&lower.as_str()) {
                    topics.insert(lower);
                }
            }
            (entities, topics)
        };

        let (k_ent, k_top) = extract(knowledge);
        let (a_ent, a_top) = extract(answer);

        if k_ent.is_empty() || a_ent.is_empty() {
            return 0.0;
        }

        let t_inter = k_top.intersection(&a_top).count();
        let t_union = k_top.union(&a_top).count();
        let topic_overlap = if t_union > 0 {
            t_inter as f64 / t_union as f64
        } else {
            0.0
        };

        if topic_overlap < 0.15 {
            return 0.0;
        }

        let e_inter = k_ent.intersection(&a_ent).count();
        let e_union = k_ent.union(&a_ent).count();
        let entity_conflict = 1.0 - (e_inter as f64 / e_union.max(1) as f64);

        if entity_conflict < 0.35 {
            return 0.0;
        }

        (topic_overlap * entity_conflict).min(1.0)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_texts_have_high_similarity() {
        let sf = KeywordSemanticField;
        assert!(sf.similarity("the cat sat on the mat", "the cat sat on the mat") > 0.95);
    }

    #[test]
    fn unrelated_texts_have_low_similarity() {
        let sf = KeywordSemanticField;
        assert!(
            sf.similarity(
                "the cat sat on the mat",
                "quantum mechanics governs subatomic particles"
            ) < 0.15
        );
    }

    #[test]
    fn negation_detected_for_opposing_polarity() {
        let sf = KeywordSemanticField;
        let score = sf.is_negation_of(
            "The universe had a beginning in time",
            "The universe has no beginning and is eternal",
        );
        assert!(
            score > 0.5,
            "Expected high contradiction score, got {score}"
        );
    }

    #[test]
    fn no_negation_for_same_polarity() {
        let sf = KeywordSemanticField;
        let score = sf.is_negation_of("The drug causes remission", "The drug causes side effects");
        assert!(score < 0.3, "Expected low contradiction score, got {score}");
    }

    #[test]
    fn category_distance_same_group_is_low() {
        let sf = KeywordSemanticField;
        assert!(sf.category_distance("Unity", "Plurality") < 0.3);
    }

    #[test]
    fn category_distance_cross_group_is_high() {
        let sf = KeywordSemanticField;
        assert!(sf.category_distance("Causality", "Necessity") > 0.5);
    }

    #[test]
    fn entity_conflict_detects_wrong_capital_city() {
        let sf = KeywordSemanticField;
        let score = sf.entity_conflict_score(
            "The capital of Australia is Canberra.",
            "Sydney is the capital of Australia.",
        );
        // Same topic (capital, Australia) but different entity (Canberra vs Sydney)
        assert!(score > 0.25, "Expected entity conflict, got {score}");
    }

    #[test]
    fn entity_conflict_low_for_consistent_answer() {
        let sf = KeywordSemanticField;
        let score = sf.entity_conflict_score(
            "The capital of Australia is Canberra.",
            "Canberra is the capital of Australia.",
        );
        // Same entities → no conflict
        assert!(
            score < 0.3,
            "Expected low conflict for consistent answer, got {score}"
        );
    }

    #[test]
    fn entity_conflict_low_for_unrelated_texts() {
        let sf = KeywordSemanticField;
        let score = sf.entity_conflict_score(
            "Photosynthesis converts light into chemical energy.",
            "Napoleon was exiled to Saint Helena.",
        );
        assert!(
            score < 0.15,
            "Expected near-zero conflict for unrelated texts, got {score}"
        );
    }
}
