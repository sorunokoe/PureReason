/// Self-Verification: Verify initial verdict for internal consistency
///
/// TRIZ Principle: Feedback + Inspection
/// After generating verdict, validate consistency and re-evaluate if needed.
///
/// Catches cases where model makes contradictory predictions across phases.
use crate::contradiction_detector;

/// Result of self-verification
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Is the verdict internally consistent?
    pub is_consistent: bool,
    /// Consistency confidence (0.0-1.0)
    pub consistency_confidence: f64,
    /// Suggested adjustment to final confidence
    pub confidence_adjustment: f64, // -1.0 to +1.0
    /// Explanation of verification result
    pub explanation: String,
}

/// Verify verdict consistency across multiple signals
///
/// # Arguments
/// * `answer_text` - The original answer being verified
/// * `phase_a_signal` - Phase A heuristic signal (0.0-1.0, higher = hallucination)
/// * `phase_b_signal` - Phase B model signal (0.0-1.0)
/// * `phase_c_signal` - Phase C contradiction signal (0.0-1.0)
/// * `final_confidence` - Current final verdict confidence
pub fn verify_consistency(
    answer_text: &str,
    phase_a_signal: f64,
    phase_b_signal: f64,
    phase_c_signal: f64,
    final_confidence: f64,
) -> VerificationResult {
    let mut issues = Vec::new();
    let mut adjustments = Vec::new();

    // Check 1: Phase signals agreement (should not conflict drastically)
    let signal_variance = compute_signal_variance([phase_a_signal, phase_b_signal, phase_c_signal]);
    if signal_variance > 0.3 {
        issues.push(format!(
            "High disagreement between phases (variance: {:.2})",
            signal_variance
        ));
    }

    // Check 2: Answer internal contradictions
    let claims = contradiction_detector::extract_claims(answer_text);
    if claims.len() >= 2 {
        let contradiction_analysis = contradiction_detector::find_contradictions(&claims);
        if !contradiction_analysis.contradictions.is_empty()
            && contradiction_analysis.overall_confidence > 0.7
        {
            // Found strong internal contradiction
            let expected_signal = 0.85;
            let signal_mismatch = (final_confidence - expected_signal).abs();
            if signal_mismatch > 0.2 {
                issues.push(format!(
                    "Internal contradiction detected (confidence: {:.2}) but final_confidence: {:.2}",
                    contradiction_analysis.overall_confidence, final_confidence
                ));
                adjustments.push(0.15); // Boost confidence slightly
            }
        }
    }

    // Check 3: Extreme confidence without strong signals
    if final_confidence > 0.85 && phase_a_signal < 0.3 && phase_b_signal < 0.3 {
        issues.push("Very high confidence without strong phase signals".to_string());
        adjustments.push(-0.15); // Reduce overconfidence
    }

    // Check 4: Very low confidence with strong signals
    if final_confidence < 0.2 && (phase_a_signal > 0.7 || phase_b_signal > 0.7) {
        issues.push("Very low confidence despite strong phase signals".to_string());
        adjustments.push(0.10);
    }

    // Check 5: Claim length vs confidence (short = should be high confidence if real)
    let word_count = answer_text.split_whitespace().count();
    if word_count < 5 && final_confidence > 0.7 && phase_b_signal < 0.4 {
        issues.push("High confidence for very short answer without model agreement".to_string());
        adjustments.push(-0.10);
    }

    let is_consistent = issues.is_empty();
    let consistency_confidence = 1.0 - (issues.len() as f64 * 0.15).min(1.0);
    let adjustment = if adjustments.is_empty() {
        0.0
    } else {
        adjustments.iter().sum::<f64>() / adjustments.len() as f64
    };

    VerificationResult {
        is_consistent,
        consistency_confidence,
        confidence_adjustment: adjustment,
        explanation: if issues.is_empty() {
            "Verdict is internally consistent across all phases".to_string()
        } else {
            format!(
                "Found {} consistency issues: {}",
                issues.len(),
                issues.join("; ")
            )
        },
    }
}

/// Compute variance of signal array
fn compute_signal_variance(signals: [f64; 3]) -> f64 {
    let mean = signals.iter().sum::<f64>() / signals.len() as f64;
    let variance = signals.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / signals.len() as f64;
    variance.sqrt() // Return std dev for interpretability
}

/// Apply self-verification adjustment to final confidence
pub fn apply_verification(current_confidence: f64, verification: &VerificationResult) -> f64 {
    if !verification.is_consistent && verification.consistency_confidence > 0.6 {
        // Apply adjustment only if fairly confident about inconsistency
        let adjusted = current_confidence + verification.confidence_adjustment;
        adjusted.clamp(0.0, 1.0)
    } else {
        current_confidence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_verdict() {
        let result = verify_consistency(
            "The Earth orbits the Sun",
            0.1,  // Phase A: low hallucination
            0.12, // Phase B: low hallucination
            0.05, // Phase C: low contradiction
            0.1,  // Final: low hallucination
        );
        assert!(result.is_consistent);
        assert!(result.consistency_confidence > 0.8);
    }

    #[test]
    fn test_high_signal_variance() {
        let result = verify_consistency(
            "The Earth orbits the Sun",
            0.1,  // Phase A: low
            0.8,  // Phase B: high (disagrees!)
            0.05, // Phase C: low
            0.5,  // Final
        );
        assert!(!result.is_consistent);
        assert!(!result.explanation.is_empty());
    }

    #[test]
    fn test_extreme_confidence_no_signals() {
        let result = verify_consistency(
            "The Earth orbits the Sun",
            0.1,  // Low
            0.2,  // Low
            0.1,  // Low
            0.95, // Very high (suspicious!)
        );
        assert!(!result.is_consistent);
    }

    #[test]
    fn test_very_low_confidence_strong_signals() {
        let result = verify_consistency(
            "The Earth orbits the Sun",
            0.8,  // High hallucination signal
            0.75, // High
            0.7,  // High
            0.1,  // Very low (suspicious!)
        );
        assert!(!result.is_consistent);
    }

    #[test]
    fn test_apply_verification_positive_adjustment() {
        let result = VerificationResult {
            is_consistent: false,
            consistency_confidence: 0.8,
            confidence_adjustment: 0.10,
            explanation: "Test".to_string(),
        };
        let adjusted = apply_verification(0.5, &result);
        assert!(adjusted > 0.5);
    }

    #[test]
    fn test_apply_verification_negative_adjustment() {
        let result = VerificationResult {
            is_consistent: false,
            consistency_confidence: 0.8,
            confidence_adjustment: -0.15,
            explanation: "Test".to_string(),
        };
        let adjusted = apply_verification(0.7, &result);
        assert!(adjusted < 0.7);
    }

    #[test]
    fn test_short_answer_no_model_agreement() {
        let result = verify_consistency(
            "Yes", // Very short
            0.1, 0.2, 0.1, 0.75, // Reasonably high confidence
        );
        assert!(!result.is_consistent);
    }
}
