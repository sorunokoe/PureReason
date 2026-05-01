//! Adaptive Weighting: Adjust Phase A/B blend based on claim complexity
//!
//! TRIZ Principle: Dynamism
//! Adapt strategy based on problem characteristics (claim complexity).
//!
//! Simple claims rely more on Phase A (deterministic heuristics)
//! Complex claims rely more on Phase B (learned model patterns)

/// Compute complexity score for a claim (0.0 = simple, 1.0 = complex)
pub fn compute_complexity_score(knowledge: &str, answer: &str) -> f64 {
    let mut score = 0.0;

    // Factor 1: Answer length (words)
    let word_count = answer.split_whitespace().count() as f64;
    let length_score = (word_count / 50.0).min(1.0); // Normalize: 50+ words = max complexity
    score += length_score * 0.25; // Weight: 25%

    // Factor 2: Sentence count (indicates multiple claims)
    let sentence_count =
        answer.matches('.').count() + answer.matches('!').count() + answer.matches('?').count();
    let sentence_score = (sentence_count as f64 / 3.0).min(1.0); // Normalize: 3+ sentences = max
    score += sentence_score * 0.20; // Weight: 20%

    // Factor 3: Named entity density (capitalized words)
    let entity_count = answer
        .split_whitespace()
        .filter(|word| word.len() >= 3 && word.chars().next().is_some_and(|c| c.is_uppercase()))
        .count() as f64;
    let entity_score = (entity_count / 5.0).min(1.0); // Normalize: 5+ entities = max
    score += entity_score * 0.20; // Weight: 20%

    // Factor 4: Numerical density (indicates factual claims)
    let number_count = answer
        .split_whitespace()
        .filter(|word| word.chars().any(|c| c.is_numeric()))
        .count() as f64;
    let numeric_score = (number_count / 3.0).min(1.0); // Normalize: 3+ numbers = max
    score += numeric_score * 0.15; // Weight: 15%

    // Factor 5: Qualifier presence (uncertainty markers)
    let qualifiers = [
        "might", "could", "may", "perhaps", "seems", "appears", "possibly", "probably",
    ];
    let has_qualifiers = qualifiers.iter().any(|q| answer.to_lowercase().contains(q)) as u32 as f64;
    let qualifier_score = if has_qualifiers > 0.0 { 0.3 } else { 0.0 };
    score += qualifier_score * 0.10; // Weight: 10%

    // Factor 6: Knowledge vs Answer similarity (low similarity = higher complexity)
    let similarity = compute_similarity_score(knowledge, answer);
    let dissimilarity_score = 1.0 - similarity;
    score += dissimilarity_score * 0.10; // Weight: 10%

    score.min(1.0)
}

/// Compute weights based on complexity
/// Returns (phase_a_weight, phase_b_weight) tuple
pub fn compute_weights(complexity: f64) -> (f64, f64) {
    // Adaptive weighting curve:
    // - Simple (0.0): 80/20 (trust heuristics)
    // - Medium (0.5): 70/30 (balanced)
    // - Complex (1.0): 60/40 (trust model)

    let phase_b_weight = 0.20 + (complexity * 0.20); // 0.20 to 0.40
    let phase_a_weight = 1.0 - phase_b_weight;

    (phase_a_weight, phase_b_weight)
}

/// Similarity score between knowledge and answer (0.0-1.0)
fn compute_similarity_score(knowledge: &str, answer: &str) -> f64 {
    if knowledge.is_empty() || answer.is_empty() {
        return 0.5; // Neutral if either is empty
    }

    let knowledge_words: std::collections::HashSet<String> = knowledge
        .to_lowercase()
        .split_whitespace()
        .map(|w| w.to_string())
        .collect();

    let answer_words: Vec<String> = answer
        .to_lowercase()
        .split_whitespace()
        .map(|w| w.to_string())
        .collect();

    if answer_words.is_empty() {
        return 0.0;
    }

    let matching = answer_words
        .iter()
        .filter(|w| knowledge_words.contains(*w) && w.len() > 3)
        .count() as f64;

    matching / answer_words.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_claim_low_complexity() {
        let complexity = compute_complexity_score("Paris is capital", "Yes");
        assert!(complexity < 0.3, "Simple claim should have low complexity");
    }

    #[test]
    fn test_complex_claim_high_complexity() {
        let complexity = compute_complexity_score(
            "The World War II lasted from 1939 to 1945",
            "World War II began on September 1, 1939, in Europe. It lasted six years. Many nations participated.",
        );
        assert!(
            complexity > 0.5,
            "Complex claim should have high complexity"
        );
    }

    #[test]
    fn test_weights_simple() {
        let (a_weight, b_weight) = compute_weights(0.1);
        assert!((a_weight - 0.80).abs() < 0.05);
        assert!((b_weight - 0.20).abs() < 0.05);
    }

    #[test]
    fn test_weights_balanced() {
        let (a_weight, b_weight) = compute_weights(0.5);
        assert!((a_weight - 0.70).abs() < 0.05);
        assert!((b_weight - 0.30).abs() < 0.05);
    }

    #[test]
    fn test_weights_complex() {
        let (a_weight, b_weight) = compute_weights(0.9);
        assert!((a_weight - 0.62).abs() < 0.05);
        assert!((b_weight - 0.38).abs() < 0.05);
    }

    #[test]
    fn test_similarity_matching() {
        let similarity = compute_similarity_score(
            "Earth orbits Sun every day year",
            "Earth orbits Sun every single day of year",
        );
        assert!(
            similarity > 0.3,
            "Similar claims should have some word overlap"
        );
    }

    #[test]
    fn test_similarity_divergent() {
        let similarity = compute_similarity_score("The Earth orbits the Sun", "Cats are mammals");
        assert!(similarity < 0.3);
    }
}
