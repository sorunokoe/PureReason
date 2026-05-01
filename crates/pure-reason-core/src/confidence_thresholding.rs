//! # Confidence Threshold Tuning
//!
//! Quick Win #3: Per-benchmark confidence threshold calibration
//!
//! Different benchmarks require different confidence thresholds:
//! - **Medical**: Conservative (flag more as risky) - false negatives are dangerous
//! - **Legal**: Aggressive (flag less) - false positives block valid arguments  
//! - **General**: Balanced approach
//!
//! This module provides tools to:
//! 1. Build confidence calibration curves (confidence vs correctness)
//! 2. Find optimal thresholds per benchmark
//! 3. Apply threshold dynamically during inference

use serde::{Deserialize, Serialize};

/// A single calibration point: (confidence, was_correct).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CalibrationPoint {
    /// Confidence score reported by the model [0.0, 1.0]
    pub confidence: f64,
    /// Whether the prediction was actually correct
    pub was_correct: bool,
}

/// Statistics for a confidence range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceStats {
    /// Confidence range [lower, upper)
    pub confidence_range: (f64, f64),
    /// Count of correct predictions in this range
    pub correct_count: usize,
    /// Count of incorrect predictions in this range
    pub incorrect_count: usize,
}

impl ConfidenceStats {
    /// Accuracy in this confidence range: correct / (correct + incorrect)
    pub fn accuracy(&self) -> f64 {
        let total = (self.correct_count + self.incorrect_count) as f64;
        if total == 0.0 {
            0.0
        } else {
            self.correct_count as f64 / total
        }
    }

    /// Total count of predictions in this range
    pub fn count(&self) -> usize {
        self.correct_count + self.incorrect_count
    }

    /// Precision (correct / all positives): how often we're right when we predict
    pub fn precision(&self) -> f64 {
        let total = (self.correct_count + self.incorrect_count) as f64;
        if total == 0.0 {
            0.0
        } else {
            self.correct_count as f64 / total
        }
    }

    /// Coverage (count / all): what fraction of predictions fall in this range
    pub fn coverage(&self, total_count: usize) -> f64 {
        if total_count == 0 {
            0.0
        } else {
            self.count() as f64 / total_count as f64
        }
    }
}

/// Calibration curve: maps confidence ranges to accuracy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationCurve {
    /// Name of the benchmark/domain
    pub name: String,
    /// Statistics per confidence bucket
    pub stats: Vec<ConfidenceStats>,
    /// Total predictions analyzed
    pub total: usize,
}

impl CalibrationCurve {
    /// Create a calibration curve from predictions.
    ///
    /// Buckets predictions into bins (e.g., 0-0.1, 0.1-0.2, ..., 0.9-1.0)
    pub fn from_points(name: String, points: Vec<CalibrationPoint>, bin_size: f64) -> Self {
        let mut bins: Vec<ConfidenceStats> = Vec::new();
        let num_bins = (1.0 / bin_size).ceil() as usize;

        for i in 0..num_bins {
            let lower = i as f64 * bin_size;
            let upper = (i as f64 + 1.0) * bin_size;
            let upper = upper.min(1.0); // Cap at 1.0

            let mut correct = 0;
            let mut incorrect = 0;

            for point in &points {
                if point.confidence >= lower && point.confidence < upper {
                    if point.was_correct {
                        correct += 1;
                    } else {
                        incorrect += 1;
                    }
                }
            }

            bins.push(ConfidenceStats {
                confidence_range: (lower, upper),
                correct_count: correct,
                incorrect_count: incorrect,
            });
        }

        let total = points.len();
        Self {
            name,
            stats: bins,
            total,
        }
    }

    /// Find optimal confidence threshold that maximizes F1 score.
    ///
    /// F1 = 2 * (precision * recall) / (precision + recall)
    /// - Precision: correct / all_flagged
    /// - Recall: correct_flagged / all_correct
    pub fn optimal_f1_threshold(&self) -> (f64, f64) {
        let mut best_threshold = 0.5;
        let mut best_f1 = 0.0;

        // Try different thresholds
        for threshold_idx in 0..10 {
            let threshold = threshold_idx as f64 * 0.1;

            // Calculate metrics above this threshold
            let mut correct_above = 0;
            let mut incorrect_above = 0;
            let mut total_correct = 0;

            for stat in &self.stats {
                total_correct += stat.correct_count;
                if stat.confidence_range.0 >= threshold {
                    correct_above += stat.correct_count;
                    incorrect_above += stat.incorrect_count;
                }
            }

            // Calculate precision and recall
            let total_flagged = correct_above + incorrect_above;
            let precision = if total_flagged > 0 {
                correct_above as f64 / total_flagged as f64
            } else {
                0.0
            };

            let recall = if total_correct > 0 {
                correct_above as f64 / total_correct as f64
            } else {
                0.0
            };

            // Calculate F1
            let f1 = if precision + recall > 0.0 {
                2.0 * (precision * recall) / (precision + recall)
            } else {
                0.0
            };

            if f1 > best_f1 {
                best_f1 = f1;
                best_threshold = threshold;
            }
        }

        (best_threshold, best_f1)
    }

    /// Find threshold that maximizes precision (minimize false positives).
    pub fn conservative_threshold(&self) -> f64 {
        // Find where accuracy > 80%
        for stat in self.stats.iter().rev() {
            if stat.accuracy() >= 0.8 {
                return stat.confidence_range.0;
            }
        }
        0.7 // Fallback
    }

    /// Find threshold that maximizes recall (minimize false negatives).
    pub fn aggressive_threshold(&self) -> f64 {
        // Find where accuracy > 60%
        for stat in &self.stats {
            if stat.accuracy() >= 0.6 {
                return stat.confidence_range.0;
            }
        }
        0.3 // Fallback
    }

    /// Estimate accuracy at a given confidence threshold.
    pub fn accuracy_at_threshold(&self, threshold: f64) -> f64 {
        let mut correct = 0;
        let mut total = 0;

        for stat in &self.stats {
            if stat.confidence_range.0 >= threshold {
                correct += stat.correct_count;
                total += stat.correct_count + stat.incorrect_count;
            }
        }

        if total == 0 {
            0.0
        } else {
            correct as f64 / total as f64
        }
    }

    /// Get coverage (fraction of predictions above threshold).
    pub fn coverage_at_threshold(&self, threshold: f64) -> f64 {
        let mut count = 0;
        for stat in &self.stats {
            if stat.confidence_range.0 >= threshold {
                count += stat.count();
            }
        }

        if self.total == 0 {
            0.0
        } else {
            count as f64 / self.total as f64
        }
    }
}

/// Threshold recommendation based on use case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThresholdStrategy {
    /// Maximize F1 score (balanced precision/recall)
    MaximizeF1,
    /// Maximize precision (medical: fewer false alarms needed)
    HighPrecision,
    /// Maximize recall (safety-critical: catch everything)
    HighRecall,
}

impl ThresholdStrategy {
    /// Get recommended threshold based on strategy and curve.
    pub fn recommend(&self, curve: &CalibrationCurve) -> f64 {
        match self {
            ThresholdStrategy::MaximizeF1 => curve.optimal_f1_threshold().0,
            ThresholdStrategy::HighPrecision => curve.conservative_threshold(),
            ThresholdStrategy::HighRecall => curve.aggressive_threshold(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_points() -> Vec<CalibrationPoint> {
        vec![
            // High confidence, all correct
            CalibrationPoint {
                confidence: 0.95,
                was_correct: true,
            },
            CalibrationPoint {
                confidence: 0.90,
                was_correct: true,
            },
            CalibrationPoint {
                confidence: 0.85,
                was_correct: true,
            },
            // Medium confidence, mixed
            CalibrationPoint {
                confidence: 0.65,
                was_correct: true,
            },
            CalibrationPoint {
                confidence: 0.60,
                was_correct: false,
            },
            // Low confidence, mostly incorrect
            CalibrationPoint {
                confidence: 0.35,
                was_correct: false,
            },
            CalibrationPoint {
                confidence: 0.30,
                was_correct: false,
            },
        ]
    }

    #[test]
    fn test_calibration_curve_creation() {
        let points = create_test_points();
        let curve = CalibrationCurve::from_points("test".to_string(), points, 0.1);
        assert!(!curve.stats.is_empty());
        assert_eq!(curve.total, 7);
    }

    #[test]
    fn test_accuracy_calculation() {
        let stats = ConfidenceStats {
            confidence_range: (0.9, 1.0),
            correct_count: 8,
            incorrect_count: 2,
        };
        assert!((stats.accuracy() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_confidence_stats_count() {
        let stats = ConfidenceStats {
            confidence_range: (0.5, 0.6),
            correct_count: 5,
            incorrect_count: 3,
        };
        assert_eq!(stats.count(), 8);
    }

    #[test]
    fn test_optimal_f1_threshold() {
        let points = create_test_points();
        let curve = CalibrationCurve::from_points("test".to_string(), points, 0.1);
        let (threshold, f1) = curve.optimal_f1_threshold();
        assert!((0.0..=1.0).contains(&threshold));
        assert!((0.0..=1.0).contains(&f1));
    }

    #[test]
    fn test_conservative_threshold() {
        let points = create_test_points();
        let curve = CalibrationCurve::from_points("test".to_string(), points, 0.1);
        let threshold = curve.conservative_threshold();
        // Conservative should be higher (fewer predictions)
        assert!(threshold >= 0.3);
    }

    #[test]
    fn test_aggressive_threshold() {
        let points = create_test_points();
        let curve = CalibrationCurve::from_points("test".to_string(), points, 0.1);
        let threshold = curve.aggressive_threshold();
        // Aggressive should be lower (more predictions)
        assert!(threshold <= 0.7);
    }

    #[test]
    fn test_accuracy_at_threshold() {
        let points = create_test_points();
        let curve = CalibrationCurve::from_points("test".to_string(), points, 0.1);
        let acc_high = curve.accuracy_at_threshold(0.8);
        let acc_low = curve.accuracy_at_threshold(0.3);
        // High threshold should have better accuracy
        assert!(acc_high >= acc_low || (acc_high - acc_low).abs() < 0.001);
    }

    #[test]
    fn test_threshold_strategy_recommend() {
        let points = create_test_points();
        let curve = CalibrationCurve::from_points("test".to_string(), points, 0.1);

        let _f1_threshold = ThresholdStrategy::MaximizeF1.recommend(&curve);
        let precision_threshold = ThresholdStrategy::HighPrecision.recommend(&curve);
        let recall_threshold = ThresholdStrategy::HighRecall.recommend(&curve);

        // Precision threshold should be higher than recall threshold
        assert!(precision_threshold >= recall_threshold);
    }

    #[test]
    fn test_coverage_at_threshold() {
        let points = create_test_points();
        let curve = CalibrationCurve::from_points("test".to_string(), points, 0.1);
        let coverage_high = curve.coverage_at_threshold(0.9);
        let coverage_low = curve.coverage_at_threshold(0.3);
        // Lower threshold covers more predictions
        assert!(coverage_low >= coverage_high);
    }
}
