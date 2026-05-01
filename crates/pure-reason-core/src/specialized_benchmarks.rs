//! # Specialized Benchmarks Integration: GSM8K, HumanEval, MMLU-Pro, ARC, DROP
//!
//! Domain-specific benchmark adapters with automatic metric collection
//! and competitive comparison tracking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// GSM8K: Grade School Math (850 grade school math word problems)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gsm8kBenchmark {
    /// Total problems in benchmark
    pub total: usize,
    /// Correct solutions
    pub correct: usize,
    /// Average steps in solution
    pub avg_steps: f64,
    /// Arithmetic errors caught
    pub arithmetic_errors: usize,
}

/// HumanEval: Code generation (164 programming problems)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanevalBenchmark {
    /// Total problems
    pub total: usize,
    /// Correct implementations
    pub correct: usize,
    /// Pass@1: chance of passing with 1 sample
    pub pass_at_1: f64,
    /// Functions with correct signature
    pub signature_correct: usize,
}

/// MMLU-Pro: Massive Multitask Language Understanding (12,605 multiple choice)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmluProBenchmark {
    /// Total questions
    pub total: usize,
    /// Correct answers
    pub correct: usize,
    /// By subject (STEM, humanities, social sciences, etc)
    pub by_subject: HashMap<String, (usize, usize)>, // (correct, total)
    /// Average confidence
    pub avg_confidence: f64,
}

/// ARC: AI2 Reasoning Challenge (7,787 science questions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcBenchmark {
    /// Easy set accuracy
    pub easy_accuracy: f64,
    /// Challenge set accuracy
    pub challenge_accuracy: f64,
    /// Correct reasoning chains identified
    pub reasoning_chains_found: usize,
    /// Hallucinations detected and prevented
    pub hallucinations_prevented: usize,
}

/// DROP: Reading comprehension + Arithmetic (10,000 questions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropBenchmark {
    /// Total passages
    pub total: usize,
    /// Questions answered correctly
    pub correct: usize,
    /// Arithmetic accuracy (on math subquestions)
    pub arithmetic_accuracy: f64,
    /// Discrete reasoning accuracy
    pub discrete_reasoning_accuracy: f64,
}

/// Competitive benchmark results (vs other systems)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitiveResult {
    /// System name (o3, R1, EVICheck, etc)
    pub system: String,
    /// Benchmark name
    pub benchmark: String,
    /// F1 score / Accuracy
    pub f1: f64,
    /// Latency (ms)
    pub latency_ms: f64,
    /// Cost per call ($)
    pub cost_per_call: f64,
}

/// Specialized benchmark tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecializedBenchmarks {
    /// GSM8K results
    pub gsm8k: Option<Gsm8kBenchmark>,
    /// HumanEval results
    pub humaneval: Option<HumanevalBenchmark>,
    /// MMLU-Pro results
    pub mmlu_pro: Option<MmluProBenchmark>,
    /// ARC results
    pub arc: Option<ArcBenchmark>,
    /// DROP results
    pub drop: Option<DropBenchmark>,
    /// Competitive comparisons
    pub competitive: Vec<CompetitiveResult>,
    /// Overall performance ranking (1-best, N-worst)
    pub overall_rank: usize,
}

impl SpecializedBenchmarks {
    /// Create new tracker
    pub fn new() -> Self {
        Self {
            gsm8k: None,
            humaneval: None,
            mmlu_pro: None,
            arc: None,
            drop: None,
            competitive: Vec::new(),
            overall_rank: 0,
        }
    }

    /// Record GSM8K result
    pub fn set_gsm8k(&mut self, benchmark: Gsm8kBenchmark) {
        self.gsm8k = Some(benchmark);
    }

    /// Record HumanEval result
    pub fn set_humaneval(&mut self, benchmark: HumanevalBenchmark) {
        self.humaneval = Some(benchmark);
    }

    /// Record MMLU-Pro result
    pub fn set_mmlu_pro(&mut self, benchmark: MmluProBenchmark) {
        self.mmlu_pro = Some(benchmark);
    }

    /// Record ARC result
    pub fn set_arc(&mut self, benchmark: ArcBenchmark) {
        self.arc = Some(benchmark);
    }

    /// Record DROP result
    pub fn set_drop(&mut self, benchmark: DropBenchmark) {
        self.drop = Some(benchmark);
    }

    /// Add competitive result
    pub fn add_competitive(&mut self, result: CompetitiveResult) {
        self.competitive.push(result);
    }

    /// Get average accuracy across all benchmarks
    pub fn average_accuracy(&self) -> f64 {
        let mut total = 0.0;
        let mut count = 0;

        if let Some(ref gsm8k) = self.gsm8k {
            total += gsm8k.correct as f64 / gsm8k.total as f64;
            count += 1;
        }
        if let Some(ref humaneval) = self.humaneval {
            total += humaneval.correct as f64 / humaneval.total as f64;
            count += 1;
        }
        if let Some(ref mmlu) = self.mmlu_pro {
            total += mmlu.correct as f64 / mmlu.total as f64;
            count += 1;
        }
        if let Some(ref arc) = self.arc {
            total += (arc.easy_accuracy + arc.challenge_accuracy) / 2.0;
            count += 1;
        }
        if let Some(ref drop) = self.drop {
            total += drop.correct as f64 / drop.total as f64;
            count += 1;
        }

        if count == 0 {
            0.0
        } else {
            total / count as f64
        }
    }

    /// Get competitive ranking vs all systems
    pub fn get_ranking(&self) -> Vec<(String, f64)> {
        let mut results = vec![("PureReason".to_string(), self.average_accuracy())];

        for comp in &self.competitive {
            results.push((comp.system.clone(), comp.f1));
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results
    }

    /// Check if we're beating a specific competitor
    pub fn beats_competitor(&self, competitor: &str) -> bool {
        let our_accuracy = self.average_accuracy();
        self.competitive
            .iter()
            .filter(|c| c.system == competitor)
            .all(|c| our_accuracy > c.f1)
    }

    /// Get competitive advantage summary
    pub fn get_advantage_summary(&self) -> String {
        let ranking = self.get_ranking();
        if ranking.is_empty() {
            return "No benchmarks configured".to_string();
        }

        if ranking[0].0 == "PureReason" {
            format!(
                "🏆 Competitive Leader! Accuracy: {:.1}%",
                ranking[0].1 * 100.0
            )
        } else {
            format!(
                "Accuracy: {:.1}% (#{} of {})",
                ranking
                    .iter()
                    .position(|r| r.0 == "PureReason")
                    .map(|p| p + 1)
                    .unwrap_or(0),
                ranking.len(),
                ranking
                    .iter()
                    .position(|r| r.0 == "PureReason")
                    .map(|_| "")
                    .unwrap_or("")
            )
        }
    }
}

impl Default for SpecializedBenchmarks {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gsm8k_creation() {
        let gsm8k = Gsm8kBenchmark {
            total: 850,
            correct: 720,
            avg_steps: 3.2,
            arithmetic_errors: 10,
        };

        assert_eq!(gsm8k.correct, 720);
        assert!(gsm8k.correct as f64 / gsm8k.total as f64 > 0.8);
    }

    #[test]
    fn test_humaneval_creation() {
        let humaneval = HumanevalBenchmark {
            total: 164,
            correct: 150,
            pass_at_1: 0.914,
            signature_correct: 160,
        };

        assert_eq!(humaneval.correct, 150);
        assert!(humaneval.pass_at_1 > 0.9);
    }

    #[test]
    fn test_specialized_benchmarks_new() {
        let sb = SpecializedBenchmarks::new();
        assert!(sb.gsm8k.is_none());
        assert!(sb.humaneval.is_none());
        assert_eq!(sb.competitive.len(), 0);
    }

    #[test]
    fn test_set_gsm8k() {
        let mut sb = SpecializedBenchmarks::new();
        let gsm8k = Gsm8kBenchmark {
            total: 850,
            correct: 750,
            avg_steps: 3.1,
            arithmetic_errors: 15,
        };

        sb.set_gsm8k(gsm8k);
        assert!(sb.gsm8k.is_some());
        assert_eq!(sb.gsm8k.as_ref().unwrap().correct, 750);
    }

    #[test]
    fn test_average_accuracy_single() {
        let mut sb = SpecializedBenchmarks::new();
        sb.set_gsm8k(Gsm8kBenchmark {
            total: 100,
            correct: 80,
            avg_steps: 2.0,
            arithmetic_errors: 5,
        });

        assert_eq!(sb.average_accuracy(), 0.8);
    }

    #[test]
    fn test_average_accuracy_multiple() {
        let mut sb = SpecializedBenchmarks::new();
        sb.set_gsm8k(Gsm8kBenchmark {
            total: 100,
            correct: 80,
            avg_steps: 2.0,
            arithmetic_errors: 5,
        });
        sb.set_humaneval(HumanevalBenchmark {
            total: 100,
            correct: 90,
            pass_at_1: 0.9,
            signature_correct: 95,
        });

        let avg = sb.average_accuracy();
        assert!(avg > 0.8 && avg < 0.95);
    }

    #[test]
    fn test_add_competitive() {
        let mut sb = SpecializedBenchmarks::new();
        sb.add_competitive(CompetitiveResult {
            system: "o3".to_string(),
            benchmark: "gsm8k".to_string(),
            f1: 0.92,
            latency_ms: 45000.0,
            cost_per_call: 0.001,
        });

        assert_eq!(sb.competitive.len(), 1);
    }

    #[test]
    fn test_get_ranking() {
        let mut sb = SpecializedBenchmarks::new();
        sb.set_gsm8k(Gsm8kBenchmark {
            total: 100,
            correct: 85,
            avg_steps: 2.0,
            arithmetic_errors: 5,
        });
        sb.add_competitive(CompetitiveResult {
            system: "o3".to_string(),
            benchmark: "gsm8k".to_string(),
            f1: 0.82,
            latency_ms: 30000.0,
            cost_per_call: 0.001,
        });

        let ranking = sb.get_ranking();
        assert_eq!(ranking[0].0, "PureReason"); // We should be first
    }

    #[test]
    fn test_beats_competitor() {
        let mut sb = SpecializedBenchmarks::new();
        sb.set_gsm8k(Gsm8kBenchmark {
            total: 100,
            correct: 85,
            avg_steps: 2.0,
            arithmetic_errors: 5,
        });
        sb.add_competitive(CompetitiveResult {
            system: "o3".to_string(),
            benchmark: "gsm8k".to_string(),
            f1: 0.80,
            latency_ms: 30000.0,
            cost_per_call: 0.001,
        });

        assert!(sb.beats_competitor("o3"));
    }

    #[test]
    fn test_advantage_summary() {
        let mut sb = SpecializedBenchmarks::new();
        sb.set_gsm8k(Gsm8kBenchmark {
            total: 100,
            correct: 90,
            avg_steps: 2.0,
            arithmetic_errors: 5,
        });

        let summary = sb.get_advantage_summary();
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_mmlu_pro_by_subject() {
        let mut by_subject = HashMap::new();
        by_subject.insert("STEM".to_string(), (80, 100));
        by_subject.insert("Humanities".to_string(), (75, 100));

        let mmlu = MmluProBenchmark {
            total: 200,
            correct: 155,
            by_subject,
            avg_confidence: 0.85,
        };

        assert_eq!(mmlu.by_subject.len(), 2);
    }

    #[test]
    fn test_arc_accuracy_tracking() {
        let arc = ArcBenchmark {
            easy_accuracy: 0.95,
            challenge_accuracy: 0.78,
            reasoning_chains_found: 450,
            hallucinations_prevented: 120,
        };

        assert!(arc.easy_accuracy > arc.challenge_accuracy);
    }

    #[test]
    fn test_drop_component_tracking() {
        let drop = DropBenchmark {
            total: 10000,
            correct: 8200,
            arithmetic_accuracy: 0.92,
            discrete_reasoning_accuracy: 0.80,
        };

        assert!(drop.arithmetic_accuracy > drop.discrete_reasoning_accuracy);
    }
}
