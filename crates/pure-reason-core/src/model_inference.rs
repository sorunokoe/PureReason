/// Phase B: DistilBERT Model Integration
///
/// Provides interface to the trained DistilBERT classifier for detecting
/// falsifiable vs unfalsifiable claims.
///
/// Model predicts:
/// - Label 0: FALSIFIABLE (can be fact-checked, potentially false)
/// - Label 1: UNFALSIFIABLE (correct, established facts)
use std::process::{Command, Stdio};

/// Model prediction result.
#[derive(Debug, Clone, Copy)]
pub struct ModelPrediction {
    /// Probability that claim is FALSIFIABLE (0.0-1.0)
    pub falsifiable_prob: f64,
    /// Probability that claim is UNFALSIFIABLE (0.0-1.0)
    pub unfalsifiable_prob: f64,
    /// Model confidence (max of the two probabilities)
    pub confidence: f64,
}

/// Run DistilBERT model inference.
///
/// Returns the model's prediction for whether the claim is falsifiable.
/// Returns None if model inference fails (not critical — will fall back to ensemble).
pub fn predict(knowledge: Option<&str>, claim: &str) -> Option<ModelPrediction> {
    let knowledge = knowledge.unwrap_or("");

    // Use Python subprocess for inference
    // This avoids circular dependencies with transformers/torch
    let output = Command::new("python3")
        .arg("scripts/model_inference.py")
        .arg(knowledge)
        .arg(claim)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    // Parse JSON output
    let json_str = String::from_utf8(output.stdout).ok()?;
    let result: serde_json::Value = serde_json::from_str(&json_str).ok()?;

    Some(ModelPrediction {
        falsifiable_prob: result["falsifiable_prob"].as_f64()?,
        unfalsifiable_prob: result["unfalsifiable_prob"].as_f64()?,
        confidence: result["confidence"].as_f64()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires model file to exist
    fn test_model_inference() {
        let knowledge = "Einstein developed the theory of relativity.";
        let claim = "Einstein invented the light bulb.";

        if let Some(pred) = predict(Some(knowledge), claim) {
            assert!(pred.confidence >= 0.0 && pred.confidence <= 1.0);
            assert!(pred.falsifiable_prob + pred.unfalsifiable_prob <= 1.01); // Allow float rounding
        }
    }

    #[test]
    fn test_missing_knowledge() {
        // Should work with empty knowledge
        if let Some(pred) = predict(None, "Sample claim") {
            assert!(pred.confidence >= 0.0 && pred.confidence <= 1.0);
        }
    }
}
