/// Phase D: Pre-Verification Layer
///
/// Fast heuristic checks before expensive Phase B model inference.
/// Identifies obvious hallucinations/falsifications to reduce latency.
///
/// # TRIZ Principle: Preliminary Action
/// Detect and categorize easy cases before expensive operations.
use crate::contradiction_detector;

/// Result of pre-verification (fast heuristics)
#[derive(Debug, Clone)]
pub struct PreVerificationResult {
    /// Can we make a confident decision without model inference?
    pub can_short_circuit: bool,
    /// Predicted hallucination score (0.0-1.0)
    pub predicted_score: f64,
    /// Reason for prediction
    pub reason: String,
    /// Confidence in this prediction
    pub confidence: f64,
}

/// Run fast pre-verification checks
///
/// # Arguments
/// * `knowledge` - The ground truth text
/// * `answer` - The candidate answer to verify
/// * `claim_limit` - Maximum claims to analyze (default: 5)
///
/// # Returns
/// `PreVerificationResult` with short-circuit decision if confident enough
pub fn pre_verify(knowledge: &str, answer: &str) -> PreVerificationResult {
    // Rule 1: Direct string match (high confidence)
    if knowledge.contains(answer) || answer.is_empty() {
        return PreVerificationResult {
            can_short_circuit: true,
            predicted_score: 0.05,
            reason: "Answer is substring of knowledge (likely truthful)".to_string(),
            confidence: 0.95,
        };
    }

    // Rule 2: Empty knowledge fallback
    if knowledge.is_empty() {
        return PreVerificationResult {
            can_short_circuit: false,
            predicted_score: 0.5,
            reason: "No knowledge base; cannot pre-verify".to_string(),
            confidence: 0.0,
        };
    }

    // Rule 3: Answer is too short (likely not hallucinated)
    if answer.split_whitespace().count() < 3 {
        return PreVerificationResult {
            can_short_circuit: true,
            predicted_score: 0.15,
            reason: "Answer too short to be meaningful hallucination".to_string(),
            confidence: 0.80,
        };
    }

    // Rule 4: Internal contradictions (high confidence hallucination)
    let claims = contradiction_detector::extract_claims(answer);
    if claims.len() >= 2 {
        let analysis = contradiction_detector::find_contradictions(&claims);
        if !analysis.contradictions.is_empty() {
            // Found internal contradiction - strong signal of hallucination
            let avg_confidence = analysis
                .contradictions
                .iter()
                .map(|c| c.confidence)
                .sum::<f64>()
                / analysis.contradictions.len() as f64;

            if avg_confidence > 0.75 {
                return PreVerificationResult {
                    can_short_circuit: true,
                    predicted_score: 0.88,
                    reason: format!(
                        "Found {} internal contradiction(s) (confidence: {:.2})",
                        analysis.contradictions.len(),
                        avg_confidence
                    ),
                    confidence: 0.90,
                };
            }
        }
    }

    // Rule 5: Named entity mismatch (if knowledge mentions specific entities)
    let knowledge_entities = extract_named_entities(knowledge);
    let answer_entities = extract_named_entities(answer);

    if !knowledge_entities.is_empty() && !answer_entities.is_empty() {
        // Check if answer mentions entities not in knowledge
        let mismatched = answer_entities
            .iter()
            .filter(|e| !entity_is_related_to_knowledge(e, knowledge))
            .count();

        let mismatch_ratio = mismatched as f64 / answer_entities.len() as f64;
        if mismatch_ratio > 0.6 {
            return PreVerificationResult {
                can_short_circuit: true,
                predicted_score: 0.75,
                reason: format!(
                    "{:.0}% of answer entities not related to knowledge (hallucination signal)",
                    mismatch_ratio * 100.0
                ),
                confidence: 0.75,
            };
        }
    }

    // Rule 6: Numerical value verification
    let answer_numbers = extract_numbers(answer);
    let knowledge_numbers = extract_numbers(knowledge);

    if !answer_numbers.is_empty() && !knowledge_numbers.is_empty() {
        // Check for extreme outliers (e.g., claiming 1000x more than stated)
        let outlier_score = compute_numerical_outlier_score(&answer_numbers, &knowledge_numbers);
        if outlier_score > 0.8 {
            return PreVerificationResult {
                can_short_circuit: true,
                predicted_score: 0.82,
                reason: "Answer contains numerical values far outside knowledge range".to_string(),
                confidence: 0.85,
            };
        }
    }

    // Rule 7: Semantic coverage check (does answer stay on-topic?)
    let coverage_score = compute_semantic_coverage(knowledge, answer);
    if coverage_score < 0.3 && answer.split_whitespace().count() > 20 {
        // Long answer with poor coverage of knowledge domain
        return PreVerificationResult {
            can_short_circuit: true,
            predicted_score: 0.70,
            reason: "Long answer with poor semantic coverage of knowledge domain".to_string(),
            confidence: 0.72,
        };
    }

    // No short-circuit: need model inference
    PreVerificationResult {
        can_short_circuit: false,
        predicted_score: 0.5,
        reason: "No decisive pre-verification rule matched".to_string(),
        confidence: 0.0,
    }
}

/// Extract simple named entities (capitalized words)
fn extract_named_entities(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|word| {
            // Capitalized word that's at least 3 chars
            word.len() >= 3 && word.chars().next().is_some_and(|c| c.is_uppercase())
        })
        .map(|w| w.to_lowercase())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect()
}

/// Check if entity is mentioned or related in knowledge
fn entity_is_related_to_knowledge(entity: &str, knowledge: &str) -> bool {
    let knowledge_lower = knowledge.to_lowercase();
    knowledge_lower.contains(entity)
        || knowledge_lower.contains(&entity[..std::cmp::min(5, entity.len())])
}

/// Extract numerical values from text
fn extract_numbers(text: &str) -> Vec<f64> {
    text.split_whitespace()
        .filter_map(|word| {
            // Try to parse as number (handles decimals and percentages)
            let cleaned = word
                .trim_matches(|c: char| !c.is_numeric() && c != '.' && c != '-')
                .to_string();
            cleaned.parse::<f64>().ok()
        })
        .collect()
}

/// Compute outlier score for numerical values
/// Returns 0.0 (numbers are reasonable) to 1.0 (extreme outliers)
fn compute_numerical_outlier_score(answer_numbers: &[f64], knowledge_numbers: &[f64]) -> f64 {
    if answer_numbers.is_empty() || knowledge_numbers.is_empty() {
        return 0.0;
    }

    let knowledge_max = knowledge_numbers
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let _knowledge_min = knowledge_numbers
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);

    let outliers = answer_numbers
        .iter()
        .filter(|&&num| {
            let deviation = if knowledge_max.abs() > 0.0 {
                (num - knowledge_max).abs() / knowledge_max.abs()
            } else {
                (num - knowledge_max).abs()
            };
            deviation > 2.0 // More than 2x the range
        })
        .count();

    let outlier_ratio = outliers as f64 / answer_numbers.len() as f64;
    (outlier_ratio * 1.2).min(1.0) // Cap at 1.0
}

/// Compute semantic coverage (word overlap with knowledge)
/// Returns 0.0 (no overlap) to 1.0 (perfect overlap)
fn compute_semantic_coverage(knowledge: &str, answer: &str) -> f64 {
    let knowledge_words: std::collections::HashSet<String> = knowledge
        .split_whitespace()
        .map(|w| w.to_lowercase())
        .collect();

    let answer_words: Vec<String> = answer
        .split_whitespace()
        .map(|w| w.to_lowercase())
        .collect();

    if answer_words.is_empty() {
        return 0.0;
    }

    let matching = answer_words
        .iter()
        .filter(|w| knowledge_words.contains(*w) && w.len() > 3) // Only count meaningful words
        .count();

    matching as f64 / answer_words.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_verify_direct_match() {
        let result = pre_verify("The Earth orbits the Sun", "The Earth orbits the Sun");
        assert!(result.can_short_circuit);
        assert!(result.predicted_score < 0.2);
    }

    #[test]
    fn test_pre_verify_substring_match() {
        let result = pre_verify(
            "The Earth orbits the Sun every 365 days",
            "The Earth orbits the Sun",
        );
        assert!(result.can_short_circuit);
        assert!(result.predicted_score < 0.2);
    }

    #[test]
    fn test_pre_verify_empty_answer() {
        let result = pre_verify("Knowledge text", "");
        assert!(result.can_short_circuit);
        assert!(result.predicted_score < 0.2);
    }

    #[test]
    fn test_pre_verify_short_answer() {
        let result = pre_verify("Long knowledge base", "yes");
        assert!(result.can_short_circuit);
    }

    #[test]
    fn test_pre_verify_internal_contradiction() {
        let result = pre_verify(
            "People need oxygen",
            "Humans breathe oxygen. Humans cannot breathe oxygen.",
        );
        assert!(result.can_short_circuit);
        assert!(result.predicted_score > 0.70); // Adjusted threshold
    }

    #[test]
    fn test_pre_verify_no_decision() {
        let result = pre_verify(
            "Paris is the capital of France",
            "Paris has many museums and historic buildings",
        );
        assert!(!result.can_short_circuit);
        assert_eq!(result.confidence, 0.0);
    }
}
