//! Confidence Calibration: Temperature Scaling for Model Predictions
//!
//! TRIZ Principle: Taking Out (Extraction)
//! Remove overconfidence signal, keep pattern recognition capability.
//!
//! The Phase B model tends to produce overconfident predictions.
//! Temperature scaling spreads the confidence distribution to improve reliability.

/// Temperature-scaled confidence calibration
/// Converts raw model probability to calibrated confidence
pub fn calibrate_confidence(raw_prob: f64, temperature: f64) -> f64 {
    if raw_prob <= 0.0 || raw_prob >= 1.0 {
        return raw_prob; // Already extreme, no need to calibrate
    }

    // Convert probability to logit space
    // logit(p) = log(p / (1-p))
    let logit = (raw_prob / (1.0 - raw_prob)).ln();

    // Apply temperature scaling
    let scaled_logit = logit / temperature;

    // Convert back to probability
    // sigmoid(z) = 1 / (1 + exp(-z))
    1.0 / (1.0 + (-scaled_logit).exp())
}

/// Adaptive temperature based on model confidence and claim characteristics
pub fn estimate_adaptive_temperature(
    raw_prob: f64,
    claim_complexity: f64,
    knowledge_length: usize,
) -> f64 {
    // Base temperature for calibration
    let mut temperature: f64 = 1.0;

    // Factor 1: High confidence signals need more scaling
    if !(0.2..=0.8).contains(&raw_prob) {
        temperature += 0.3; // Increase temperature for extreme predictions
    }

    // Factor 2: Complex claims should be less confident
    if claim_complexity > 0.7 {
        temperature += 0.2;
    }

    // Factor 3: Short knowledge base should reduce confidence
    if knowledge_length < 50 {
        temperature += 0.25;
    }

    temperature.min(2.0) // Cap at 2.0 to avoid over-smoothing
}

/// Apply full calibration pipeline
pub fn apply_calibration(
    raw_prob: f64,
    claim_complexity: f64,
    knowledge_length: usize,
) -> (f64, String) {
    let temperature = estimate_adaptive_temperature(raw_prob, claim_complexity, knowledge_length);
    let calibrated = calibrate_confidence(raw_prob, temperature);

    let reason = format!(
        "Calibration: {:.3} → {:.3} (T={:.2}, complexity={:.2}, knowledge_len={})",
        raw_prob, calibrated, temperature, claim_complexity, knowledge_length
    );

    (calibrated, reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calibrate_extreme_high() {
        let calibrated = calibrate_confidence(0.95, 1.5);
        assert!(calibrated < 0.95, "High confidence should be reduced");
        assert!(calibrated > 0.8, "Should still be high confidence");
    }

    #[test]
    fn test_calibrate_extreme_low() {
        let calibrated = calibrate_confidence(0.05, 1.5);
        assert!(calibrated > 0.05, "Low confidence should be increased");
        assert!(calibrated < 0.2, "Should still be low confidence");
    }

    #[test]
    fn test_calibrate_mid_range() {
        let calibrated = calibrate_confidence(0.5, 1.5);
        assert!(
            (calibrated - 0.5).abs() < 0.01,
            "Mid-range should be stable"
        );
    }

    #[test]
    fn test_temperature_high_confidence() {
        let temp = estimate_adaptive_temperature(0.9, 0.5, 200);
        assert!(temp > 1.0, "High confidence should increase temperature");
    }

    #[test]
    fn test_temperature_complex_claim() {
        let temp = estimate_adaptive_temperature(0.6, 0.8, 200);
        assert!(temp > 1.0, "Complex claim should increase temperature");
    }

    #[test]
    fn test_temperature_short_knowledge() {
        let temp = estimate_adaptive_temperature(0.6, 0.5, 30);
        assert!(temp > 1.0, "Short knowledge should increase temperature");
    }

    #[test]
    fn test_full_calibration_pipeline() {
        let (calibrated, _reason) = apply_calibration(0.85, 0.6, 150);
        assert!(
            calibrated <= 0.85,
            "Calibrated should not exceed raw probability for high values"
        );
    }
}
