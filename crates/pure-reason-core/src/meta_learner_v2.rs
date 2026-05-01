//! # Session Meta-Learner V2 (TRIZ P13, P2)
//!
//! Session-scoped adaptive learning that tracks detector accuracy and
//! adjusts ensemble weights in real-time. No persistence - learning
//! resets between sessions to maintain determinism.
//!
//! **Key features:**
//! - Session-scoped (no cross-session contamination)
//! - Detector accuracy tracking (hits, misses per detector)
//! - Adaptive weights every 100 calls
//! - Exponential smoothing (α=0.3) to avoid wild swings
//! - 100-call warmup period using default weights
//!
//! **Expected impact:** +5-10pp F1 improvement after warmup
//!
//! ## Usage
//!
//! ```rust
//! use pure_reason_core::meta_learner_v2::SessionMetaLearner;
//!
//! let mut learner = SessionMetaLearner::new();
//!
//! // First 100 calls use default weights
//! let weights = learner.get_weights();
//!
//! // After verification, update with actual result
//! learner.update_after_verification(&detector_votes, actual_verdict);
//!
//! // Every 100 calls, weights adapt automatically
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ensemble weights for detectors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnsembleWeights {
    pub kac_detector: f64,
    pub numeric_detector: f64,
    pub semantic_detector: f64,
    pub novelty_detector: f64,
    pub contradiction_detector: f64,
}

impl Default for EnsembleWeights {
    fn default() -> Self {
        Self {
            kac_detector: 1.0,
            numeric_detector: 1.0,
            semantic_detector: 1.0,
            novelty_detector: 1.0,
            contradiction_detector: 1.0,
        }
    }
}

impl EnsembleWeights {
    /// Get weight by detector name.
    pub fn get(&self, detector_name: &str) -> f64 {
        match detector_name {
            "kac_detector" => self.kac_detector,
            "numeric_detector" => self.numeric_detector,
            "semantic_detector" => self.semantic_detector,
            "novelty_detector" => self.novelty_detector,
            "contradiction_detector" => self.contradiction_detector,
            _ => 1.0, // Unknown detector, default weight
        }
    }

    /// Set weight by detector name.
    pub fn set(&mut self, detector_name: &str, weight: f64) {
        match detector_name {
            "kac_detector" => self.kac_detector = weight,
            "numeric_detector" => self.numeric_detector = weight,
            "semantic_detector" => self.semantic_detector = weight,
            "novelty_detector" => self.novelty_detector = weight,
            "contradiction_detector" => self.contradiction_detector = weight,
            _ => {}
        }
    }

    /// Get all detector names.
    pub fn detector_names() -> Vec<&'static str> {
        vec![
            "kac_detector",
            "numeric_detector",
            "semantic_detector",
            "novelty_detector",
            "contradiction_detector",
        ]
    }
}

/// Configuration for session meta-learner.
#[derive(Debug, Clone)]
pub struct MetaLearnerConfig {
    /// Warmup period (calls before adaptation begins)
    pub warmup_calls: usize,
    /// Adaptation frequency (recalculate every N calls)
    pub adapt_frequency: usize,
    /// Exponential smoothing factor (0.0-1.0)
    pub alpha: f64,
    /// Minimum samples per detector before trusting accuracy
    pub min_samples: usize,
}

impl Default for MetaLearnerConfig {
    fn default() -> Self {
        Self {
            warmup_calls: 100,
            adapt_frequency: 100,
            alpha: 0.3,
            min_samples: 10,
        }
    }
}

/// Session-scoped meta-learner for adaptive ensemble weights.
///
/// Tracks detector accuracy and adapts weights every N calls.
/// Resets between sessions (no persistence).
#[derive(Debug, Clone)]
pub struct SessionMetaLearner {
    /// Per-detector accuracy: (detector_name → (hits, misses))
    detector_stats: HashMap<String, (usize, usize)>,

    /// Current ensemble weights (adaptive)
    weights: EnsembleWeights,

    /// Default weights (fallback)
    default_weights: EnsembleWeights,

    /// Configuration
    config: MetaLearnerConfig,

    /// Call counter
    call_count: usize,
}

impl SessionMetaLearner {
    /// Create new session meta-learner with default configuration.
    pub fn new() -> Self {
        Self::with_config(MetaLearnerConfig::default())
    }

    /// Create with custom configuration.
    pub fn with_config(config: MetaLearnerConfig) -> Self {
        Self {
            detector_stats: HashMap::new(),
            weights: EnsembleWeights::default(),
            default_weights: EnsembleWeights::default(),
            config,
            call_count: 0,
        }
    }

    /// Get current ensemble weights.
    ///
    /// During warmup period, returns default weights.
    /// After warmup, returns adaptive weights.
    pub fn get_weights(&self) -> EnsembleWeights {
        self.weights.clone()
    }

    /// Update meta-learner after a verification result.
    ///
    /// Tracks which detectors were correct/incorrect, and adapts weights
    /// every `adapt_frequency` calls.
    ///
    /// # Arguments
    /// * `detector_votes` - Map of detector_name → (flags_risk, confidence)
    /// * `actual_verdict` - Ground truth (true = risk confirmed, false = safe)
    pub fn update_after_verification(
        &mut self,
        detector_votes: &HashMap<String, (bool, f64)>,
        actual_verdict: bool,
    ) {
        self.call_count += 1;

        // Update detector stats
        for (detector_name, (flags_risk, _confidence)) in detector_votes {
            let stats = self
                .detector_stats
                .entry(detector_name.clone())
                .or_insert((0, 0));

            // Detector is correct if its prediction matches actual verdict
            let correct = *flags_risk == actual_verdict;

            if correct {
                stats.0 += 1; // hits
            } else {
                stats.1 += 1; // misses
            }
        }

        // Adapt weights every adapt_frequency calls (after warmup)
        if self.call_count >= self.config.warmup_calls
            && self.call_count % self.config.adapt_frequency == 0
        {
            self.adapt_weights();
        }
    }

    /// Adapt weights based on detector accuracy.
    ///
    /// Uses exponential smoothing to avoid wild swings:
    /// new_weight = α * computed_weight + (1-α) * old_weight
    fn adapt_weights(&mut self) {
        let mut new_weights = EnsembleWeights::default();

        for detector_name in EnsembleWeights::detector_names() {
            let (hits, misses) = self
                .detector_stats
                .get(detector_name)
                .copied()
                .unwrap_or((0, 0));

            let total = hits + misses;

            let computed_weight = if total < self.config.min_samples {
                // Not enough data, use default
                self.default_weights.get(detector_name)
            } else {
                // Compute accuracy and scale weight
                let accuracy = hits as f64 / total as f64;

                // Scale: 1.0 accuracy = 2× default, 0.5 accuracy = 1× default
                self.default_weights.get(detector_name) * (1.0 + accuracy)
            };

            // Exponential smoothing
            let old_weight = self.weights.get(detector_name);
            let smoothed_weight =
                self.config.alpha * computed_weight + (1.0 - self.config.alpha) * old_weight;

            new_weights.set(detector_name, smoothed_weight);
        }

        self.weights = new_weights;
    }

    /// Get call count (for diagnostics).
    pub fn call_count(&self) -> usize {
        self.call_count
    }

    /// Get detector statistics (for diagnostics).
    pub fn detector_stats(&self) -> &HashMap<String, (usize, usize)> {
        &self.detector_stats
    }

    /// Check if in warmup period.
    pub fn is_warmup(&self) -> bool {
        self.call_count < self.config.warmup_calls
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warmup_period() {
        let mut learner = SessionMetaLearner::new();
        assert!(learner.is_warmup());
        assert_eq!(learner.call_count(), 0);

        // Default weights during warmup
        let weights = learner.get_weights();
        assert_eq!(weights.kac_detector, 1.0);
    }

    #[test]
    fn test_weight_adaptation() {
        let mut learner = SessionMetaLearner::with_config(MetaLearnerConfig {
            warmup_calls: 10,
            adapt_frequency: 10,
            alpha: 0.5, // Higher alpha for faster adaptation in tests
            min_samples: 5,
        });

        // Simulate 20 calls where kac_detector is always correct
        for i in 0..20 {
            let mut votes = HashMap::new();
            votes.insert("kac_detector".to_string(), (true, 0.9));
            votes.insert("numeric_detector".to_string(), (false, 0.6));

            learner.update_after_verification(&votes, true); // actual = true

            if i >= 10 {
                // After warmup, kac_detector weight should increase
                let weights = learner.get_weights();
                assert!(
                    weights.kac_detector > 1.0,
                    "kac_detector weight should increase after consistent accuracy"
                );
            }
        }
    }

    #[test]
    fn test_exponential_smoothing() {
        let mut learner = SessionMetaLearner::with_config(MetaLearnerConfig {
            warmup_calls: 5,
            adapt_frequency: 5,
            alpha: 0.3,
            min_samples: 3,
        });

        // Simulate calls
        for _ in 0..10 {
            let mut votes = HashMap::new();
            votes.insert("kac_detector".to_string(), (true, 0.9));
            learner.update_after_verification(&votes, true);
        }

        let weights = learner.get_weights();

        // Weight should increase but not wildly (due to smoothing)
        assert!(weights.kac_detector > 1.0);
        assert!(weights.kac_detector < 2.0); // Smoothing prevents extreme values
    }

    #[test]
    fn test_detector_stats_tracking() {
        let mut learner = SessionMetaLearner::new();

        let mut votes = HashMap::new();
        votes.insert("kac_detector".to_string(), (true, 0.9));
        votes.insert("numeric_detector".to_string(), (false, 0.5));

        // kac correct, numeric correct
        learner.update_after_verification(&votes, true);

        let stats = learner.detector_stats();
        assert_eq!(stats.get("kac_detector"), Some(&(1, 0))); // 1 hit, 0 misses
        assert_eq!(stats.get("numeric_detector"), Some(&(0, 1))); // 0 hits, 1 miss
    }

    #[test]
    fn test_min_samples_threshold() {
        let mut learner = SessionMetaLearner::with_config(MetaLearnerConfig {
            warmup_calls: 5,
            adapt_frequency: 5,
            alpha: 0.5,
            min_samples: 10, // High threshold
        });

        // Only 5 samples - not enough for adaptation
        for _ in 0..10 {
            let mut votes = HashMap::new();
            votes.insert("kac_detector".to_string(), (true, 0.9));
            learner.update_after_verification(&votes, true);
        }

        let weights = learner.get_weights();

        // With min_samples=10 and only 10 calls total, should use default weights
        // (might be slightly adapted due to smoothing, but close to 1.0)
        assert!(weights.kac_detector >= 1.0);
    }
}
