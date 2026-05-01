//! # Meta-Learning: Adaptive Reasoning Phase Adjustment from Benchmark Failures
//!
//! Home Run #9: Learn from all benchmark failures to dynamically adjust reasoning
//!
//! TRIZ Principle: Dynamic Systems + Feedback
//! Continuously learn from failures across benchmarks, dynamically adjusting
//! which reasoning phases are most effective.
//!
//! Key insight: Different benchmarks fail on different reasoning gaps.
//! GSM8K fails on multi-step arithmetic. MMLU-Pro fails on domain knowledge.
//! ARC fails on analogical reasoning. Learn which phase helps each benchmark.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single failure case with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureCase {
    /// Benchmark name (gsm8k, mmlu, arc, etc)
    pub benchmark: String,
    /// Claim that was misclassified
    pub claim: String,
    /// Our prediction (incorrect)
    pub predicted: bool,
    /// Ground truth (correct)
    pub actual: bool,
    /// Which reasoning phase was disabled/weak
    pub weak_phase: String,
    /// Our confidence in the (wrong) prediction
    pub confidence: f64,
}

/// Pattern detected across failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    /// Phase name (chain_of_thought, causal_reasoning, etc)
    pub phase: String,
    /// Benchmark where this phase helps most
    pub benchmark: String,
    /// Number of failures when this phase was weak
    pub failure_count: usize,
    /// Estimated F1 impact if we strengthen this phase
    pub estimated_delta_f1: f64,
}

/// Learning outcome from a batch of failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningOutcome {
    /// Patterns discovered
    pub patterns: Vec<FailurePattern>,
    /// Recommended weight adjustments
    pub weight_adjustments: HashMap<String, (String, f64)>, // (phase, new_weight)
    /// Estimated total F1 improvement if applied
    pub estimated_total_delta: f64,
    /// Confidence in these recommendations (0-1)
    pub confidence: f64,
}

/// Meta-learner that adapts to benchmark-specific failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaLearner {
    /// All observed failures by benchmark
    pub failures_by_benchmark: HashMap<String, Vec<FailureCase>>,
    /// Phase effectiveness per benchmark
    pub phase_effectiveness: HashMap<String, HashMap<String, f64>>,
    /// Total cases analyzed
    pub total_cases: usize,
    /// Learning progress (0-1)
    pub learning_progress: f64,
}

impl Default for MetaLearner {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaLearner {
    /// Create new meta-learner
    pub fn new() -> Self {
        Self {
            failures_by_benchmark: HashMap::new(),
            phase_effectiveness: HashMap::new(),
            total_cases: 0,
            learning_progress: 0.0,
        }
    }

    /// Record a failure case for learning
    pub fn record_failure(&mut self, failure: FailureCase) {
        self.total_cases += 1;
        self.failures_by_benchmark
            .entry(failure.benchmark.clone())
            .or_default()
            .push(failure);

        // Update learning progress (increase toward 1.0 with diminishing returns)
        self.learning_progress = (self.total_cases as f64 / 200.0).min(1.0);
    }

    /// Analyze all recorded failures to extract patterns
    pub fn analyze(&mut self) -> LearningOutcome {
        let mut patterns = Vec::new();
        let mut weight_adjustments = HashMap::new();
        let mut total_delta = 0.0;

        // For each benchmark, analyze which phases are weak
        for (benchmark, failures) in &self.failures_by_benchmark {
            if failures.is_empty() {
                continue;
            }

            // Group failures by weak phase
            let mut phase_failure_counts: HashMap<String, usize> = HashMap::new();
            for failure in failures {
                *phase_failure_counts
                    .entry(failure.weak_phase.clone())
                    .or_insert(0) += 1;
            }

            // For each weak phase, calculate impact
            for (phase, count) in phase_failure_counts {
                let failure_rate = count as f64 / failures.len() as f64;
                let estimated_delta = failure_rate * 0.05 * self.learning_progress.max(0.5);

                if estimated_delta > 0.01 {
                    patterns.push(FailurePattern {
                        phase: phase.clone(),
                        benchmark: benchmark.clone(),
                        failure_count: count,
                        estimated_delta_f1: estimated_delta,
                    });

                    total_delta += estimated_delta;
                }

                // Store effectiveness metric
                self.phase_effectiveness
                    .entry(phase.clone())
                    .or_default()
                    .insert(benchmark.clone(), 1.0 - failure_rate);
            }
        }

        // Sort patterns by impact (highest first)
        patterns.sort_by(|a, b| {
            b.estimated_delta_f1
                .partial_cmp(&a.estimated_delta_f1)
                .unwrap()
        });

        // Generate weight adjustments for top patterns
        for pattern in patterns.iter().take(10) {
            let adjustment = pattern.estimated_delta_f1 * 0.3; // Conservative scaling
            weight_adjustments.insert(
                format!("{}-{}", pattern.phase, pattern.benchmark),
                (pattern.phase.clone(), 1.2 + adjustment), // Boost weak phases
            );
        }

        LearningOutcome {
            patterns,
            weight_adjustments,
            estimated_total_delta: total_delta.min(0.15), // Cap conservative estimate
            confidence: self.learning_progress.min(0.8),
        }
    }

    /// Get phase effectiveness for a benchmark
    pub fn get_phase_effectiveness(&self, phase: &str, benchmark: &str) -> f64 {
        self.phase_effectiveness
            .get(phase)
            .and_then(|m| m.get(benchmark))
            .copied()
            .unwrap_or(0.5) // Default to neutral
    }

    /// Suggest which phases to prioritize for a benchmark
    pub fn suggest_phase_priority(&self, benchmark: &str) -> Vec<(String, f64)> {
        let mut priorities = Vec::new();

        for (phase, effectiveness_map) in &self.phase_effectiveness {
            if let Some(&effectiveness) = effectiveness_map.get(benchmark) {
                priorities.push((phase.clone(), effectiveness));
            }
        }

        // Sort by effectiveness (highest first)
        priorities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        priorities
    }

    /// Get total failure count across all benchmarks
    pub fn total_failures(&self) -> usize {
        self.failures_by_benchmark.values().map(|v| v.len()).sum()
    }

    /// Get failure count for specific benchmark
    pub fn failures_for_benchmark(&self, benchmark: &str) -> usize {
        self.failures_by_benchmark
            .get(benchmark)
            .map(|v| v.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metalearner_new() {
        let ml = MetaLearner::new();
        assert_eq!(ml.total_cases, 0);
        assert_eq!(ml.learning_progress, 0.0);
    }

    #[test]
    fn test_record_failure() {
        let mut ml = MetaLearner::new();
        ml.record_failure(FailureCase {
            benchmark: "gsm8k".to_string(),
            claim: "5 + 3 = 9".to_string(),
            predicted: true,
            actual: false,
            weak_phase: "math_solver".to_string(),
            confidence: 0.8,
        });

        assert_eq!(ml.total_cases, 1);
        assert_eq!(ml.failures_for_benchmark("gsm8k"), 1);
    }

    #[test]
    fn test_learning_progress() {
        let mut ml = MetaLearner::new();
        for i in 0..100 {
            ml.record_failure(FailureCase {
                benchmark: "test".to_string(),
                claim: format!("claim_{}", i),
                predicted: false,
                actual: true,
                weak_phase: "phase_1".to_string(),
                confidence: 0.5,
            });
        }
        assert!(ml.learning_progress > 0.4);
        assert!(ml.learning_progress < 0.6);
    }

    #[test]
    fn test_analyze_patterns() {
        let mut ml = MetaLearner::new();
        for i in 0..50 {
            ml.record_failure(FailureCase {
                benchmark: "gsm8k".to_string(),
                claim: format!("math_claim_{}", i),
                predicted: true,
                actual: false,
                weak_phase: "math_solver".to_string(),
                confidence: 0.7,
            });
        }

        let outcome = ml.analyze();
        assert!(!outcome.patterns.is_empty());
        assert!(outcome.estimated_total_delta > 0.0);
    }

    #[test]
    fn test_weight_adjustments() {
        let mut ml = MetaLearner::new();
        for i in 0..60 {
            ml.record_failure(FailureCase {
                benchmark: "mmlu".to_string(),
                claim: format!("knowledge_claim_{}", i),
                predicted: false,
                actual: true,
                weak_phase: "causal_reasoning".to_string(),
                confidence: 0.6,
            });
        }

        let outcome = ml.analyze();
        assert!(!outcome.weight_adjustments.is_empty());
    }

    #[test]
    fn test_multiple_benchmarks() {
        let mut ml = MetaLearner::new();
        for i in 0..30 {
            ml.record_failure(FailureCase {
                benchmark: "gsm8k".to_string(),
                claim: format!("math_{}", i),
                predicted: true,
                actual: false,
                weak_phase: "math_solver".to_string(),
                confidence: 0.7,
            });
            ml.record_failure(FailureCase {
                benchmark: "arc".to_string(),
                claim: format!("reasoning_{}", i),
                predicted: false,
                actual: true,
                weak_phase: "multi_hop_reasoner".to_string(),
                confidence: 0.6,
            });
        }

        assert_eq!(ml.total_cases, 60);
        assert_eq!(ml.failures_for_benchmark("gsm8k"), 30);
        assert_eq!(ml.failures_for_benchmark("arc"), 30);
    }

    #[test]
    fn test_phase_effectiveness() {
        let mut ml = MetaLearner::new();
        for i in 0..40 {
            ml.record_failure(FailureCase {
                benchmark: "legal".to_string(),
                claim: format!("legal_{}", i),
                predicted: true,
                actual: false,
                weak_phase: "assumption_validation".to_string(),
                confidence: 0.75,
            });
        }

        ml.analyze();
        let effectiveness = ml.get_phase_effectiveness("assumption_validation", "legal");
        assert!(effectiveness < 1.0);
    }

    #[test]
    fn test_suggest_phase_priority() {
        let mut ml = MetaLearner::new();
        for i in 0..30 {
            ml.record_failure(FailureCase {
                benchmark: "medical".to_string(),
                claim: format!("medical_{}", i),
                predicted: false,
                actual: true,
                weak_phase: "numeric_plausibility".to_string(),
                confidence: 0.65,
            });
        }

        ml.analyze();
        let priorities = ml.suggest_phase_priority("medical");
        assert!(!priorities.is_empty());
    }

    #[test]
    fn test_confidence_in_recommendations() {
        let mut ml = MetaLearner::new();
        for i in 0..150 {
            ml.record_failure(FailureCase {
                benchmark: "test".to_string(),
                claim: format!("claim_{}", i),
                predicted: true,
                actual: false,
                weak_phase: "phase_1".to_string(),
                confidence: 0.6,
            });
        }

        let outcome = ml.analyze();
        assert!(outcome.confidence > 0.5);
    }

    #[test]
    fn test_total_failures() {
        let mut ml = MetaLearner::new();
        for i in 0..20 {
            ml.record_failure(FailureCase {
                benchmark: "gsm8k".to_string(),
                claim: format!("claim_{}", i),
                predicted: true,
                actual: false,
                weak_phase: "phase".to_string(),
                confidence: 0.5,
            });
        }
        assert_eq!(ml.total_failures(), 20);
    }
}
