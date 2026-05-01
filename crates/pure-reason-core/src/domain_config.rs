//! # Domain-Specific Configuration
//!
//! Quick Win #1: Hyperparameter Sweep Per Domain
//!
//! This module provides domain-specific tuning parameters for the ensemble verifier.
//! Different domains have different hallucination patterns:
//! - **Medical**: Emphasis on numeric plausibility, causal reasoning (mechanisms)
//! - **Legal**: Emphasis on logical consistency, precedent matching
//! - **Finance**: Emphasis on numeric plausibility, risk assessment
//! - **Science**: Emphasis on causal reasoning, evidence requirements
//! - **Code**: Emphasis on logic flow, syntax validation
//!
//! Each domain gets optimized weights for ensemble detectors, confidence thresholds,
//! and which Tier 2 phases to enable.

use serde::{Deserialize, Serialize};

/// Domain classification for input routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Domain {
    /// Medical: drug interactions, diagnoses, procedures
    Medical,
    /// Legal: contracts, precedents, arguments
    Legal,
    /// Finance: market analysis, risk assessment, calculations
    Finance,
    /// Science: physics, biology, chemistry, mechanisms
    Science,
    /// Code: software, algorithms, logic flow
    Code,
    /// General fallback
    General,
}

impl Domain {
    /// Parse domain from text (check for keywords).
    pub fn infer_from_text(text: &str) -> Self {
        let lower = text.to_lowercase();

        if lower.contains("medical")
            || lower.contains("diagnosis")
            || lower.contains("drug")
            || lower.contains("patient")
            || lower.contains("doctor")
            || lower.contains("hospital")
            || lower.contains("disease")
            || lower.contains("treatment")
        {
            Domain::Medical
        } else if lower.contains("legal")
            || lower.contains("law")
            || lower.contains("court")
            || lower.contains("contract")
            || lower.contains("attorney")
            || lower.contains("statute")
            || lower.contains("precedent")
            || lower.contains("trial")
        {
            Domain::Legal
        } else if lower.contains("finance")
            || lower.contains("stock")
            || lower.contains("market")
            || lower.contains("investment")
            || lower.contains("return")
            || lower.contains("portfolio")
            || lower.contains("profit")
            || lower.contains("revenue")
        {
            Domain::Finance
        } else if lower.contains("physics")
            || lower.contains("chemistry")
            || lower.contains("biology")
            || lower.contains("quantum")
            || lower.contains("relativity")
            || lower.contains("molecule")
            || lower.contains("enzyme")
            || lower.contains("mechanism")
        {
            Domain::Science
        } else if lower.contains("code")
            || lower.contains("function")
            || lower.contains("algorithm")
            || lower.contains("variable")
            || lower.contains("compile")
            || lower.contains("debug")
            || lower.contains("syntax")
            || lower.contains("loop")
        {
            Domain::Code
        } else {
            Domain::General
        }
    }
}

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Domain::Medical => write!(f, "medical"),
            Domain::Legal => write!(f, "legal"),
            Domain::Finance => write!(f, "finance"),
            Domain::Science => write!(f, "science"),
            Domain::Code => write!(f, "code"),
            Domain::General => write!(f, "general"),
        }
    }
}

/// Ensemble detector weights per domain.
/// Each detector gets a weight [0.0, 2.0]; higher = more influential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnsembleWeights {
    /// Semantic Drift Detector weight (catches contextual shifts)
    pub semantic_drift: f64,
    /// Formal Logic Checker weight (validates reasoning chains)
    pub formal_logic: f64,
    /// Numeric Domain Detector weight (scientific/medical constants)
    pub numeric_plausibility: f64,
    /// Novelty Detector weight (flags new entities)
    pub novelty: f64,
    /// Contradiction Synthesizer weight (cross-checks knowledge base)
    pub contradiction: f64,
}

impl EnsembleWeights {
    /// Baseline weights (uniform across all detectors).
    pub fn baseline() -> Self {
        Self {
            semantic_drift: 1.0,
            formal_logic: 1.0,
            numeric_plausibility: 1.0,
            novelty: 1.0,
            contradiction: 1.0,
        }
    }

    /// Medical domain: Emphasize numeric plausibility + causal reasoning.
    pub fn medical() -> Self {
        Self {
            semantic_drift: 1.0,
            formal_logic: 1.2, // Higher: logical reasoning (dosing, interactions)
            numeric_plausibility: 1.8, // Highest: numeric values critical
            novelty: 1.0,
            contradiction: 1.3, // Higher: contraindications matter
        }
    }

    /// Legal domain: Emphasize logical consistency + precedent matching.
    pub fn legal() -> Self {
        Self {
            semantic_drift: 1.3,       // Higher: precise language matters
            formal_logic: 1.9,         // Highest: logic is essential
            numeric_plausibility: 1.0, // Lower: less numeric
            novelty: 1.2,              // Higher: precedent novelty matters
            contradiction: 1.7,        // High: contradictions matter
        }
    }

    /// Finance domain: Emphasize numeric plausibility + risk assessment.
    pub fn finance() -> Self {
        Self {
            semantic_drift: 1.1,
            formal_logic: 1.3,         // Higher: logic chains (if returns > X%)
            numeric_plausibility: 1.9, // Highest: numbers are critical
            novelty: 1.0,
            contradiction: 1.4, // Higher: contradictions in risk
        }
    }

    /// Science domain: Emphasize causal reasoning + evidence.
    pub fn science() -> Self {
        Self {
            semantic_drift: 1.2,
            formal_logic: 1.6,         // High: mechanisms require logic
            numeric_plausibility: 1.5, // High: constants matter
            novelty: 1.3,              // Higher: novel claims need evidence
            contradiction: 1.1,
        }
    }

    /// Code domain: Emphasize logic flow + syntax validation.
    pub fn code() -> Self {
        Self {
            semantic_drift: 1.0,
            formal_logic: 1.8,         // Highest: logic is code
            numeric_plausibility: 1.2, // Higher: off-by-one errors
            novelty: 1.1,
            contradiction: 1.3, // Higher: logical contradictions in code
        }
    }

    /// Get weights for a domain.
    pub fn for_domain(domain: Domain) -> Self {
        match domain {
            Domain::Medical => Self::medical(),
            Domain::Legal => Self::legal(),
            Domain::Finance => Self::finance(),
            Domain::Science => Self::science(),
            Domain::Code => Self::code(),
            Domain::General => Self::baseline(),
        }
    }
}

/// Confidence threshold per domain (when to flag something as risky).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceThresholds {
    /// Flag as risky if hallucination_probability >= threshold
    pub hallucination_flag: f64,
    /// Skip expensive phases if confidence is too low
    pub skip_phase_threshold: f64,
}

impl ConfidenceThresholds {
    /// Baseline thresholds (conservative).
    pub fn baseline() -> Self {
        Self {
            hallucination_flag: 0.5,
            skip_phase_threshold: 0.05,
        }
    }

    /// Medical: Lower threshold (false negatives are dangerous).
    pub fn medical() -> Self {
        Self {
            hallucination_flag: 0.40, // More aggressive
            skip_phase_threshold: 0.10,
        }
    }

    /// Legal: High threshold (false positives are damaging).
    pub fn legal() -> Self {
        Self {
            hallucination_flag: 0.60, // Conservative
            skip_phase_threshold: 0.15,
        }
    }

    /// Finance: Moderate threshold.
    pub fn finance() -> Self {
        Self {
            hallucination_flag: 0.50,
            skip_phase_threshold: 0.10,
        }
    }

    /// Science: Moderate threshold (evidence-driven).
    pub fn science() -> Self {
        Self {
            hallucination_flag: 0.48,
            skip_phase_threshold: 0.08,
        }
    }

    /// Code: High threshold (logic-driven, clearer right/wrong).
    pub fn code() -> Self {
        Self {
            hallucination_flag: 0.45,
            skip_phase_threshold: 0.05,
        }
    }

    /// Get thresholds for a domain.
    pub fn for_domain(domain: Domain) -> Self {
        match domain {
            Domain::Medical => Self::medical(),
            Domain::Legal => Self::legal(),
            Domain::Finance => Self::finance(),
            Domain::Science => Self::science(),
            Domain::Code => Self::code(),
            Domain::General => Self::baseline(),
        }
    }
}

/// Which Tier 2 phases are enabled per domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier2Phases {
    /// Chain of Thought (always enabled for debugging)
    pub chain_of_thought: bool,
    /// Uncertainty Quantification
    pub uncertainty_quantification: bool,
    /// Counterargument Synthesis (for dialectical domains)
    pub counterargument_synthesis: bool,
    /// Causal Reasoning (for mechanism-heavy domains)
    pub causal_reasoning: bool,
    /// Assumption Validation (for logic-heavy domains)
    pub assumption_validation: bool,
}

impl Tier2Phases {
    /// All phases enabled.
    pub fn all() -> Self {
        Self {
            chain_of_thought: true,
            uncertainty_quantification: true,
            counterargument_synthesis: true,
            causal_reasoning: true,
            assumption_validation: true,
        }
    }

    /// Medical: Emphasize causal reasoning (mechanisms) + assumptions.
    pub fn medical() -> Self {
        Self {
            chain_of_thought: true,
            uncertainty_quantification: true,
            counterargument_synthesis: false, // Less relevant
            causal_reasoning: true,           // Mechanisms matter
            assumption_validation: true,
        }
    }

    /// Legal: Emphasize counterarguments + assumptions.
    pub fn legal() -> Self {
        Self {
            chain_of_thought: true,
            uncertainty_quantification: true,
            counterargument_synthesis: true, // Both sides matter
            causal_reasoning: false,         // Less relevant
            assumption_validation: true,
        }
    }

    /// Finance: Emphasize uncertainty + assumptions.
    pub fn finance() -> Self {
        Self {
            chain_of_thought: true,
            uncertainty_quantification: true, // Risk quantification
            counterargument_synthesis: false,
            causal_reasoning: true, // Causality in markets
            assumption_validation: true,
        }
    }

    /// Science: Emphasize causal reasoning + assumptions.
    pub fn science() -> Self {
        Self {
            chain_of_thought: true,
            uncertainty_quantification: true,
            counterargument_synthesis: false,
            causal_reasoning: true, // Mechanisms
            assumption_validation: true,
        }
    }

    /// Code: Emphasize logical assumptions + causal.
    pub fn code() -> Self {
        Self {
            chain_of_thought: true,
            uncertainty_quantification: true, // Execution uncertainty
            counterargument_synthesis: false,
            causal_reasoning: true,      // Call chains
            assumption_validation: true, // Logic assumptions
        }
    }

    /// Get phases for a domain.
    pub fn for_domain(domain: Domain) -> Self {
        match domain {
            Domain::Medical => Self::medical(),
            Domain::Legal => Self::legal(),
            Domain::Finance => Self::finance(),
            Domain::Science => Self::science(),
            Domain::Code => Self::code(),
            Domain::General => Self::all(),
        }
    }
}

/// Complete configuration for a domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    pub domain: Domain,
    pub ensemble_weights: EnsembleWeights,
    pub confidence_thresholds: ConfidenceThresholds,
    pub tier2_phases: Tier2Phases,
}

impl DomainConfig {
    /// Get configuration for a specific domain.
    pub fn for_domain(domain: Domain) -> Self {
        Self {
            domain,
            ensemble_weights: EnsembleWeights::for_domain(domain),
            confidence_thresholds: ConfidenceThresholds::for_domain(domain),
            tier2_phases: Tier2Phases::for_domain(domain),
        }
    }

    /// Get config by inferring domain from text.
    pub fn infer_from_text(text: &str) -> Self {
        let domain = Domain::infer_from_text(text);
        Self::for_domain(domain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_inference() {
        assert_eq!(
            Domain::infer_from_text("patient medication"),
            Domain::Medical
        );
        assert_eq!(Domain::infer_from_text("legal contract"), Domain::Legal);
        assert_eq!(
            Domain::infer_from_text("stock market returns"),
            Domain::Finance
        );
        assert_eq!(Domain::infer_from_text("quantum physics"), Domain::Science);
        assert_eq!(Domain::infer_from_text("function loop"), Domain::Code);
    }

    #[test]
    fn test_medical_weights() {
        let weights = EnsembleWeights::medical();
        assert!(weights.numeric_plausibility > weights.formal_logic);
        assert!(weights.numeric_plausibility > weights.semantic_drift);
    }

    #[test]
    fn test_legal_weights() {
        let weights = EnsembleWeights::legal();
        assert!(weights.formal_logic >= weights.contradiction);
        assert!(weights.formal_logic > weights.numeric_plausibility);
    }

    #[test]
    fn test_domain_config_consistency() {
        for domain in &[
            Domain::Medical,
            Domain::Legal,
            Domain::Finance,
            Domain::Science,
            Domain::Code,
        ] {
            let config = DomainConfig::for_domain(*domain);
            assert_eq!(config.domain, *domain);
            // All weights should be between 0.5 and 2.5
            let weights = &config.ensemble_weights;
            assert!(weights.semantic_drift >= 0.5 && weights.semantic_drift <= 2.5);
            assert!(weights.formal_logic >= 0.5 && weights.formal_logic <= 2.5);
            assert!(weights.numeric_plausibility >= 0.5 && weights.numeric_plausibility <= 2.5);
        }
    }

    #[test]
    fn test_confidence_thresholds() {
        let medical = ConfidenceThresholds::medical();
        let legal = ConfidenceThresholds::legal();
        // Medical should be more aggressive (lower threshold = more flags)
        assert!(medical.hallucination_flag < legal.hallucination_flag);
    }

    #[test]
    fn test_tier2_phases_per_domain() {
        let medical = Tier2Phases::medical();
        let legal = Tier2Phases::legal();

        // Medical emphasizes causal
        assert!(medical.causal_reasoning);
        assert!(!medical.counterargument_synthesis);

        // Legal emphasizes counterarguments
        assert!(legal.counterargument_synthesis);
        assert!(!legal.causal_reasoning);
    }
}
