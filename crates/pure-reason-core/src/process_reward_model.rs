// Process Reward Model - Phase 3.5.2
// Scores each reasoning phase independently and combines via domain-specific weights

use crate::domain_config::Domain;

/// Score for a single reasoning phase
#[derive(Debug, Clone)]
pub struct PhaseScore {
    pub phase_index: usize,
    pub score: f64, // 0.0-1.0
}

/// Result of process-level reward scoring
#[derive(Debug, Clone)]
pub struct ProcessScore {
    pub phase_scores: Vec<f64>, // One score per phase (0-1)
    pub weighted_score: f64,    // Final weighted score (0-1)
    pub domain: Domain,
}

/// Domain-specific process reward model
pub struct ProcessRewardModel {
    domain: Domain,
    phase_weights: Vec<f64>, // Importance weights for each phase
}

impl ProcessRewardModel {
    /// Create reward model for a specific domain
    pub fn for_domain(domain: Domain) -> Self {
        let phase_weights = Self::get_domain_weights(domain);
        ProcessRewardModel {
            domain,
            phase_weights,
        }
    }

    /// Get domain-specific phase importance weights
    fn get_domain_weights(domain: Domain) -> Vec<f64> {
        match domain {
            // Medical: Causal mechanisms are most important (30%), followed by assumptions (25%)
            Domain::Medical => vec![0.20, 0.15, 0.10, 0.30, 0.25],

            // Legal: Assumptions crucial (35%), counterargument essential (25%)
            Domain::Legal => vec![0.15, 0.10, 0.25, 0.15, 0.35],

            // Finance: Numeric accuracy (phase 1), assumptions (30%), causal (20%)
            Domain::Finance => vec![0.25, 0.10, 0.15, 0.20, 0.30],

            // Science: Causal mechanisms (35%), evidence (uncertainty 20%)
            Domain::Science => vec![0.20, 0.20, 0.15, 0.35, 0.10],

            // Code: Syntax (phase 1), causal (how it works), assumptions
            Domain::Code => vec![0.30, 0.10, 0.10, 0.25, 0.25],

            // General: Equal weights
            Domain::General => vec![0.20, 0.20, 0.20, 0.20, 0.20],
        }
    }

    /// Score the full reasoning process across all phases
    pub fn score_reasoning_process(&self, phase_scores_input: &[f64]) -> ProcessScore {
        let phase_scores = if phase_scores_input.len() == 5 {
            phase_scores_input.to_vec()
        } else {
            // Default scores if not 5 phases
            vec![0.75; 5]
        };

        // Compute weighted average
        let weighted_score: f64 = phase_scores
            .iter()
            .zip(&self.phase_weights)
            .map(|(score, weight)| score * weight)
            .sum::<f64>()
            / self.phase_weights.iter().sum::<f64>();

        ProcessScore {
            phase_scores,
            weighted_score,
            domain: self.domain,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_medical_weights_prioritize_causal() {
        let weights = ProcessRewardModel::get_domain_weights(Domain::Medical);
        // Causal (index 3) should be highest weight
        assert_eq!(weights[3], 0.30);
        // Assumption (index 4) should be second
        assert_eq!(weights[4], 0.25);
    }

    #[test]
    fn test_legal_weights_prioritize_assumptions() {
        let weights = ProcessRewardModel::get_domain_weights(Domain::Legal);
        // Assumption (index 4) should be highest for legal
        assert_eq!(weights[4], 0.35);
        // Counterargument (index 2) important for legal
        assert_eq!(weights[2], 0.25);
    }

    #[test]
    fn test_finance_weights_sum_to_one() {
        let weights = ProcessRewardModel::get_domain_weights(Domain::Finance);
        let sum: f64 = weights.iter().sum();
        assert!(
            (sum - 1.0).abs() < 0.001,
            "Weights should sum to 1.0, got {}",
            sum
        );
    }

    #[test]
    fn test_all_domain_weights_normalized() {
        for domain in &[
            Domain::Medical,
            Domain::Legal,
            Domain::Finance,
            Domain::Science,
            Domain::Code,
            Domain::General,
        ] {
            let weights = ProcessRewardModel::get_domain_weights(*domain);
            let sum: f64 = weights.iter().sum();
            assert!(
                (sum - 1.0).abs() < 0.001,
                "Domain {:?} weights don't sum to 1.0, got {}",
                domain,
                sum
            );
        }
    }
}
