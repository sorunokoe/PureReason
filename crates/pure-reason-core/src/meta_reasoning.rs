// Meta-Reasoning Self-Critique - Phase 3.5.3
// Analyzes own reasoning and routes through alternative paths if needed

use crate::domain_config::Domain;
use std::collections::HashMap;

/// Result of self-critique and routing decision
#[derive(Debug, Clone)]
pub struct MetaRoutingResult {
    pub confidence: f64,
    pub reasoning_path: String, // "standard", "alternative_uncertainty", etc.
    pub retry_needed: bool,
    pub explanation: String,
}

/// Quality analysis of reasoning
#[derive(Debug, Clone)]
pub struct QualityAnalysis {
    pub overall_quality: f64,
    pub phase_qualities: Vec<f64>,
    pub weakest_phase: usize,
    pub quality_factor: f64, // Multiplier for original confidence
}

/// Meta-reasoning system for self-critique and routing
pub struct MetaReasoner {
    #[allow(dead_code)]
    domain: Domain,
    phase_performance_history: HashMap<String, Vec<bool>>, // path -> outcomes
    confidence_threshold: f64,
}

impl MetaReasoner {
    /// Create meta-reasoner for a domain
    pub fn for_domain(domain: Domain) -> Self {
        MetaReasoner {
            domain,
            phase_performance_history: HashMap::new(),
            confidence_threshold: 0.65,
        }
    }

    /// Perform self-critique and route through alternative paths if needed
    pub fn self_critique_and_route(&mut self, initial_confidence: f64) -> MetaRoutingResult {
        // 1. Analyze own reasoning quality
        let quality = self.analyze_reasoning_quality();

        // 2. Compute effective confidence
        let effective_confidence = (initial_confidence * quality.quality_factor).clamp(0.0, 1.0);

        // 3. Decide if retry is needed
        if effective_confidence >= self.confidence_threshold {
            return MetaRoutingResult {
                confidence: effective_confidence,
                reasoning_path: "standard".to_string(),
                retry_needed: false,
                explanation: format!(
                    "Reasoning quality: {:.1}%, confidence sufficient",
                    quality.overall_quality * 100.0
                ),
            };
        }

        // 4. If not, identify alternative routing
        let weak_phase = quality.weakest_phase;
        let alt_path = self.select_alternative_path(weak_phase);

        MetaRoutingResult {
            confidence: effective_confidence,
            reasoning_path: format!("alternative_{}", weak_phase),
            retry_needed: true,
            explanation: format!(
                "Phase {} quality low ({:.1}%), routing through alternative: {}",
                weak_phase,
                quality.phase_qualities[weak_phase] * 100.0,
                alt_path
            ),
        }
    }

    /// Analyze quality of each reasoning phase
    fn analyze_reasoning_quality(&self) -> QualityAnalysis {
        let mut phase_qualities = Vec::new();

        // Score each phase based on domain-specific criteria
        for idx in 0..5 {
            let quality = self.score_phase_quality(idx);
            phase_qualities.push(quality);
        }

        // Find weakest phase
        let weakest_phase = phase_qualities
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);

        // Compute quality factor (multiplicative adjustment to confidence)
        let overall_quality: f64 =
            phase_qualities.iter().sum::<f64>() / phase_qualities.len() as f64;
        let quality_factor = overall_quality; // Directly proportional

        QualityAnalysis {
            overall_quality,
            phase_qualities,
            weakest_phase,
            quality_factor,
        }
    }

    /// Score quality of a single phase
    fn score_phase_quality(&self, phase_index: usize) -> f64 {
        // Base scores by domain; in production would parse actual phase content
        match self.domain {
            Domain::Medical => match phase_index {
                0 => 0.80, // CoT
                1 => 0.65, // Uncertainty (often weak)
                2 => 0.70, // Counterargument
                3 => 0.85, // Causal (should be strong)
                4 => 0.80, // Assumption
                _ => 0.70,
            },
            Domain::Legal => match phase_index {
                0 => 0.75, // CoT
                1 => 0.60, // Uncertainty (precedent strength)
                2 => 0.85, // Counterargument (important)
                3 => 0.70, // Causal
                4 => 0.88, // Assumption (critical)
                _ => 0.75,
            },
            Domain::Finance => match phase_index {
                0 => 0.80, // CoT
                1 => 0.70, // Uncertainty (risk)
                2 => 0.75, // Counterargument
                3 => 0.80, // Causal (economic drivers)
                4 => 0.85, // Assumption (financial assumptions critical)
                _ => 0.78,
            },
            Domain::Science => match phase_index {
                0 => 0.80, // CoT
                1 => 0.75, // Uncertainty (confidence intervals)
                2 => 0.70, // Counterargument
                3 => 0.88, // Causal (mechanism)
                4 => 0.72, // Assumption
                _ => 0.77,
            },
            Domain::Code => match phase_index {
                0 => 0.82, // CoT (logic explanation)
                1 => 0.60, // Uncertainty (weak for code)
                2 => 0.65, // Counterargument (edge cases)
                3 => 0.80, // Causal (control flow)
                4 => 0.78, // Assumption (preconditions)
                _ => 0.73,
            },
            Domain::General => 0.75,
        }
    }

    /// Select alternative execution path based on weak phase
    fn select_alternative_path(&self, weak_phase: usize) -> String {
        match weak_phase {
            0 => "strengthen_cot_with_examples".to_string(),
            1 => "move_assumption_earlier".to_string(),
            2 => "boost_counterargument".to_string(),
            3 => "reinforce_mechanism".to_string(),
            4 => "deeper_assumption_validation".to_string(),
            _ => "retry_with_emphasis".to_string(),
        }
    }

    /// Learn from outcome to improve future routing
    pub fn learn_from_outcome(&mut self, reasoning_path: &str, was_correct: bool) {
        // Record outcome for this path
        self.phase_performance_history
            .entry(reasoning_path.to_string())
            .or_default()
            .push(was_correct);

        // Adaptive threshold: if many successes, we can be more aggressive
        let outcomes = self.phase_performance_history.values().collect::<Vec<_>>();
        if !outcomes.is_empty() {
            let total_outcomes: usize = outcomes.iter().map(|o| o.len()).sum();
            let successes: usize = outcomes
                .iter()
                .map(|o| o.iter().filter(|&&x| x).count())
                .sum();

            if total_outcomes > 10 {
                let success_rate = successes as f64 / total_outcomes as f64;

                // If we're being too conservative, lower threshold
                if success_rate > 0.85 {
                    self.confidence_threshold = (self.confidence_threshold * 0.98).max(0.50);
                }
                // If we're making errors, tighten threshold
                else if success_rate < 0.65 {
                    self.confidence_threshold = (self.confidence_threshold * 1.02).min(0.80);
                }
            }
        }
    }

    /// Get current confidence threshold
    pub fn get_confidence_threshold(&self) -> f64 {
        self.confidence_threshold
    }

    /// Get performance history for a reasoning path
    pub fn get_path_success_rate(&self, path: &str) -> Option<f64> {
        self.phase_performance_history.get(path).map(|outcomes| {
            let successes = outcomes.iter().filter(|&&x| x).count();
            successes as f64 / outcomes.len() as f64
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_reasoner_detects_quality_issues() {
        let mut reasoner = MetaReasoner::for_domain(Domain::Medical);

        // GIVEN: Low initial confidence
        let initial_confidence = 0.50;

        // WHEN: Perform self-critique
        let result = reasoner.self_critique_and_route(initial_confidence);

        // THEN: Retry triggered if quality low enough
        if result.confidence < reasoner.confidence_threshold {
            assert!(result.retry_needed);
        }
    }

    #[test]
    fn test_meta_reasoner_learning() {
        let mut reasoner = MetaReasoner::for_domain(Domain::Legal);

        // GIVEN: Train on successful path
        for _ in 0..5 {
            reasoner.learn_from_outcome("standard", true);
        }

        // WHEN: Check success rate
        let success_rate = reasoner.get_path_success_rate("standard");

        // THEN: Should be 100% after 5 successes
        assert_eq!(success_rate, Some(1.0));
    }

    #[test]
    fn test_meta_reasoner_adaptive_threshold() {
        let mut reasoner = MetaReasoner::for_domain(Domain::Finance);
        let initial_threshold = reasoner.get_confidence_threshold();

        // GIVEN: Many successful outcomes
        for _ in 0..15 {
            reasoner.learn_from_outcome("standard", true);
        }

        // WHEN: Check threshold after learning
        let new_threshold = reasoner.get_confidence_threshold();

        // THEN: Threshold should lower (more aggressive)
        assert!(new_threshold < initial_threshold);
    }

    #[test]
    fn test_meta_reasoner_tightens_after_failures() {
        let mut reasoner = MetaReasoner::for_domain(Domain::Science);
        let initial_threshold = reasoner.get_confidence_threshold();

        // GIVEN: Many failed outcomes
        for _ in 0..15 {
            reasoner.learn_from_outcome("standard", false);
        }

        // WHEN: Check threshold after failures
        let new_threshold = reasoner.get_confidence_threshold();

        // THEN: Threshold should tighten (more conservative)
        assert!(new_threshold > initial_threshold);
    }

    #[test]
    fn test_alternative_path_selection() {
        let reasoner = MetaReasoner::for_domain(Domain::Medical);

        // GIVEN: Weak uncertainty phase (index 1)
        let alt_path = reasoner.select_alternative_path(1);

        // THEN: Should suggest moving assumption earlier
        assert_eq!(alt_path, "move_assumption_earlier".to_string());
    }
}
