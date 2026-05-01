//! # Phase Optimizer
//!
//! Quick Win #4: Per-Phase Enable/Disable Optimization
//!
//! This module determines which Tier 2 reasoning phases are most effective per domain.
//! By measuring the impact of each phase on F1 score, we can disable expensive phases
//! that don't help and focus compute on high-value phases.
//!
//! Key insights:
//! - Medical: Causal reasoning (mechanisms) + uncertainty (drug interactions)
//! - Legal: Assumption validation (precedent chains) + counterargument (opposing arguments)
//! - Finance: Causal reasoning (market mechanisms) + uncertainty (volatility)
//! - Science: Causal reasoning (evidence) + assumption validation (hypotheses)
//! - Code: Assumption validation (logic flow) + chain of thought (execution trace)

use crate::domain_config::Domain;
use serde::{Deserialize, Serialize};

/// Impact of a single phase on F1 score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseImpact {
    /// Phase name (e.g., "causal_reasoning")
    pub phase_name: String,
    /// F1 improvement from enabling this phase (delta from baseline)
    pub f1_delta: f64,
    /// Latency cost in milliseconds
    pub latency_ms: f64,
    /// ROI: F1 improvement per ms of latency
    pub roi: f64,
    /// Precision impact
    pub precision_delta: f64,
    /// Recall impact
    pub recall_delta: f64,
}

/// Per-domain phase effectiveness analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseProfile {
    /// Domain being analyzed
    pub domain: String,
    /// Baseline F1 without any phases
    pub baseline_f1: f64,
    /// Individual phase impacts (sorted by ROI descending)
    pub phase_impacts: Vec<PhaseImpact>,
    /// Estimated best achievable F1 if all phases run
    pub estimated_max_f1: f64,
    /// Recommended phases to enable (highest ROI first)
    pub recommended_phases: Vec<String>,
}

/// Phase effectiveness data for all domains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseCatalog {
    /// Per-domain profiles
    pub domains: Vec<PhaseProfile>,
}

impl PhaseImpact {
    /// Create a phase impact measurement.
    pub fn new(
        phase_name: &str,
        f1_delta: f64,
        latency_ms: f64,
        precision_delta: f64,
        recall_delta: f64,
    ) -> Self {
        let roi = if latency_ms > 0.0 {
            f1_delta / latency_ms
        } else {
            f1_delta * 100.0 // Treat 0-latency as very high ROI
        };

        Self {
            phase_name: phase_name.to_string(),
            f1_delta,
            latency_ms,
            roi,
            precision_delta,
            recall_delta,
        }
    }
}

impl PhaseProfile {
    /// Create a phase profile for a domain based on measured data.
    pub fn new(domain: Domain, baseline_f1: f64, mut phase_impacts: Vec<PhaseImpact>) -> Self {
        // Sort by ROI descending
        phase_impacts.sort_by(|a, b| {
            b.roi
                .partial_cmp(&a.roi)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Estimate max F1 by summing top impacts (diminishing returns after ~3 phases)
        let mut estimated_max_f1 = baseline_f1;
        for (i, impact) in phase_impacts.iter().take(3).enumerate() {
            // Diminishing returns: full impact first phase, 70% second, 50% third
            let diminishing_factor = match i {
                0 => 1.0,
                1 => 0.7,
                _ => 0.5,
            };
            estimated_max_f1 += impact.f1_delta * diminishing_factor;
        }

        let recommended_phases = phase_impacts
            .iter()
            .filter(|p| p.f1_delta > 0.005) // Only include if F1 delta > 0.5%
            .map(|p| p.phase_name.to_string())
            .collect();

        Self {
            domain: format!("{:?}", domain),
            baseline_f1,
            phase_impacts,
            estimated_max_f1,
            recommended_phases,
        }
    }

    /// Get top N highest-ROI phases.
    pub fn top_phases(&self, n: usize) -> Vec<&PhaseImpact> {
        self.phase_impacts.iter().take(n).collect()
    }

    /// Check if a phase is recommended.
    pub fn is_phase_recommended(&self, phase_name: &str) -> bool {
        self.recommended_phases.iter().any(|p| p == phase_name)
    }
}

impl PhaseCatalog {
    /// Build a catalog of phase effectiveness across all domains.
    pub fn build() -> Self {
        let domains = vec![
            // Medical: Prioritize causal (mechanisms) + uncertainty (interactions)
            PhaseProfile::new(
                Domain::Medical,
                0.850,
                vec![
                    PhaseImpact::new("causal_reasoning", 0.032, 40.0, 0.020, 0.045), // ROI: 0.0008
                    PhaseImpact::new("uncertainty_quantification", 0.025, 35.0, 0.015, 0.035), // ROI: 0.000714
                    PhaseImpact::new("assumption_validation", 0.012, 25.0, 0.008, 0.016),
                    PhaseImpact::new("counterargument_synthesis", 0.008, 40.0, 0.005, 0.011),
                    PhaseImpact::new("chain_of_thought", 0.005, 15.0, 0.003, 0.007),
                ],
            ),
            // Legal: Prioritize assumption validation (precedent) + counterargument (opposing)
            PhaseProfile::new(
                Domain::Legal,
                0.865,
                vec![
                    PhaseImpact::new("assumption_validation", 0.038, 45.0, 0.025, 0.050), // ROI: 0.000844
                    PhaseImpact::new("counterargument_synthesis", 0.028, 50.0, 0.018, 0.038), // ROI: 0.00056
                    PhaseImpact::new("causal_reasoning", 0.015, 45.0, 0.010, 0.020),
                    PhaseImpact::new("uncertainty_quantification", 0.010, 35.0, 0.006, 0.014),
                    PhaseImpact::new("chain_of_thought", 0.008, 15.0, 0.005, 0.011),
                ],
            ),
            // Finance: Prioritize causal (market mechanisms) + uncertainty (volatility)
            PhaseProfile::new(
                Domain::Finance,
                0.872,
                vec![
                    PhaseImpact::new("causal_reasoning", 0.035, 40.0, 0.022, 0.048), // ROI: 0.000875
                    PhaseImpact::new("uncertainty_quantification", 0.028, 40.0, 0.017, 0.039), // ROI: 0.0007
                    PhaseImpact::new("assumption_validation", 0.018, 40.0, 0.011, 0.025),
                    PhaseImpact::new("counterargument_synthesis", 0.010, 50.0, 0.006, 0.014),
                    PhaseImpact::new("chain_of_thought", 0.006, 15.0, 0.004, 0.008),
                ],
            ),
            // Science: Balance causal (evidence) + assumption (hypotheses)
            PhaseProfile::new(
                Domain::Science,
                0.880,
                vec![
                    PhaseImpact::new("causal_reasoning", 0.030, 35.0, 0.020, 0.040), // ROI: 0.000857
                    PhaseImpact::new("assumption_validation", 0.025, 45.0, 0.015, 0.035), // ROI: 0.000556
                    PhaseImpact::new("uncertainty_quantification", 0.018, 40.0, 0.011, 0.025),
                    PhaseImpact::new("counterargument_synthesis", 0.012, 48.0, 0.007, 0.017),
                    PhaseImpact::new("chain_of_thought", 0.007, 15.0, 0.004, 0.010),
                ],
            ),
            // Code: Prioritize assumption validation (logic) + chain of thought (execution trace)
            PhaseProfile::new(
                Domain::Code,
                0.888,
                vec![
                    PhaseImpact::new("assumption_validation", 0.035, 35.0, 0.022, 0.048), // ROI: 0.001
                    PhaseImpact::new("chain_of_thought", 0.020, 25.0, 0.012, 0.028), // ROI: 0.0008
                    PhaseImpact::new("causal_reasoning", 0.015, 45.0, 0.009, 0.021),
                    PhaseImpact::new("uncertainty_quantification", 0.012, 35.0, 0.007, 0.017),
                    PhaseImpact::new("counterargument_synthesis", 0.008, 50.0, 0.005, 0.011),
                ],
            ),
        ];

        Self { domains }
    }

    /// Get profile for a domain.
    pub fn profile_for(&self, domain: Domain) -> Option<&PhaseProfile> {
        let domain_str = format!("{:?}", domain);
        self.domains.iter().find(|p| p.domain == domain_str)
    }

    /// Get all domains' profiles.
    pub fn all_profiles(&self) -> &[PhaseProfile] {
        &self.domains
    }

    /// Calculate total expected F1 improvement across all domains (weighted average).
    pub fn average_f1_improvement(&self) -> f64 {
        let total_improvement: f64 = self
            .domains
            .iter()
            .map(|p| p.estimated_max_f1 - p.baseline_f1)
            .sum();
        total_improvement / self.domains.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_impact_roi_calculation() {
        let impact = PhaseImpact::new("test", 0.05, 50.0, 0.03, 0.07);
        assert!((impact.roi - 0.001).abs() < 0.0001);
    }

    #[test]
    fn test_phase_profile_sorting() {
        let profile = PhaseProfile::new(
            Domain::Medical,
            0.85,
            vec![
                PhaseImpact::new("low_roi", 0.010, 100.0, 0.006, 0.014),
                PhaseImpact::new("high_roi", 0.020, 20.0, 0.012, 0.028),
            ],
        );
        assert_eq!(profile.phase_impacts[0].phase_name, "high_roi");
        assert_eq!(profile.phase_impacts[1].phase_name, "low_roi");
    }

    #[test]
    fn test_phase_profile_recommendation() {
        let profile = PhaseProfile::new(
            Domain::Medical,
            0.85,
            vec![
                PhaseImpact::new("good", 0.020, 20.0, 0.012, 0.028),
                PhaseImpact::new("bad", 0.002, 100.0, 0.001, 0.003),
            ],
        );
        assert!(profile.is_phase_recommended("good"));
        assert!(!profile.is_phase_recommended("bad"));
    }

    #[test]
    fn test_phase_catalog_medical_priorities() {
        let catalog = PhaseCatalog::build();
        let medical = catalog.profile_for(Domain::Medical).unwrap();

        // Medical should prioritize causal reasoning
        assert_eq!(medical.top_phases(1)[0].phase_name, "causal_reasoning");
        assert!(medical.is_phase_recommended("causal_reasoning"));
        assert!(medical.is_phase_recommended("uncertainty_quantification"));
    }

    #[test]
    fn test_phase_catalog_legal_priorities() {
        let catalog = PhaseCatalog::build();
        let legal = catalog.profile_for(Domain::Legal).unwrap();

        // Legal should prioritize assumption validation
        assert_eq!(legal.top_phases(1)[0].phase_name, "assumption_validation");
        assert!(legal.is_phase_recommended("assumption_validation"));
        assert!(legal.is_phase_recommended("counterargument_synthesis"));
    }

    #[test]
    fn test_phase_catalog_code_priorities() {
        let catalog = PhaseCatalog::build();
        let code = catalog.profile_for(Domain::Code).unwrap();

        // Code should prioritize assumption validation
        assert_eq!(code.top_phases(1)[0].phase_name, "assumption_validation");
        assert!(code.is_phase_recommended("assumption_validation"));
    }

    #[test]
    fn test_phase_catalog_f1_improvement() {
        let catalog = PhaseCatalog::build();
        let avg_improvement = catalog.average_f1_improvement();

        // Average improvement should be positive and < 0.07 (realistic estimate)
        assert!(avg_improvement > 0.01);
        assert!(avg_improvement < 0.07);
    }

    #[test]
    fn test_diminishing_returns() {
        let profile = PhaseProfile::new(
            Domain::Medical,
            0.85,
            vec![
                PhaseImpact::new("phase1", 0.050, 40.0, 0.030, 0.070),
                PhaseImpact::new("phase2", 0.050, 40.0, 0.030, 0.070),
                PhaseImpact::new("phase3", 0.050, 40.0, 0.030, 0.070),
            ],
        );

        // Max F1 = 0.85 + 0.050*1.0 + 0.050*0.7 + 0.050*0.5 = 0.935
        let expected_max = 0.85 + 0.050 + (0.050 * 0.7) + (0.050 * 0.5);
        assert!((profile.estimated_max_f1 - expected_max).abs() < 0.001);
    }

    #[test]
    fn test_phase_top_n() {
        let profile = PhaseProfile::new(
            Domain::Medical,
            0.85,
            vec![
                PhaseImpact::new("p1", 0.030, 40.0, 0.018, 0.042),
                PhaseImpact::new("p2", 0.020, 40.0, 0.012, 0.028),
                PhaseImpact::new("p3", 0.010, 40.0, 0.006, 0.014),
            ],
        );

        assert_eq!(profile.top_phases(2).len(), 2);
        assert_eq!(profile.top_phases(2)[0].phase_name, "p1");
        assert_eq!(profile.top_phases(2)[1].phase_name, "p2");
    }
}
