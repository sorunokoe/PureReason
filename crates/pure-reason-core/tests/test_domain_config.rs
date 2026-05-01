//! Tests for domain-specific configuration (Quick Win #1)
//!
//! Verifies that domain configurations are properly tuned and
//! can be applied to the ensemble verifier.

use pure_reason_core::domain_config::{
    ConfidenceThresholds, Domain, DomainConfig, EnsembleWeights,
};

#[test]
fn test_all_domains_have_configs() {
    let domains = [
        Domain::Medical,
        Domain::Legal,
        Domain::Finance,
        Domain::Science,
        Domain::Code,
        Domain::General,
    ];

    for domain in &domains {
        let config = DomainConfig::for_domain(*domain);
        assert_eq!(config.domain, *domain, "Domain mismatch for {:?}", domain);

        // Verify weights are in valid range
        let w = &config.ensemble_weights;
        assert!(
            (0.5..=2.5).contains(&w.semantic_drift),
            "Invalid semantic_drift for {:?}",
            domain
        );
        assert!(
            (0.5..=2.5).contains(&w.formal_logic),
            "Invalid formal_logic for {:?}",
            domain
        );
        assert!(
            (0.5..=2.5).contains(&w.numeric_plausibility),
            "Invalid numeric_plausibility for {:?}",
            domain
        );
        assert!(
            (0.5..=2.5).contains(&w.novelty),
            "Invalid novelty for {:?}",
            domain
        );
        assert!(
            (0.5..=2.5).contains(&w.contradiction),
            "Invalid contradiction for {:?}",
            domain
        );

        // Verify thresholds are valid probabilities
        let t = &config.confidence_thresholds;
        assert!(
            (0.0..=1.0).contains(&t.hallucination_flag),
            "Invalid hallucination_flag for {:?}",
            domain
        );
        assert!(
            (0.0..=1.0).contains(&t.skip_phase_threshold),
            "Invalid skip_phase_threshold for {:?}",
            domain
        );
    }
}

#[test]
fn test_medical_domain_emphasizes_numeric() {
    let config = DomainConfig::for_domain(Domain::Medical);
    let w = &config.ensemble_weights;
    assert!(w.numeric_plausibility > w.semantic_drift);
    assert!(w.numeric_plausibility > w.novelty);
}

#[test]
fn test_legal_domain_emphasizes_logic() {
    let config = DomainConfig::for_domain(Domain::Legal);
    let w = &config.ensemble_weights;
    assert!(w.formal_logic > w.numeric_plausibility);
    assert!(w.formal_logic > w.novelty);
}

#[test]
fn test_medical_enables_causal_reasoning() {
    let config = DomainConfig::for_domain(Domain::Medical);
    assert!(config.tier2_phases.causal_reasoning);
    assert!(!config.tier2_phases.counterargument_synthesis);
}

#[test]
fn test_legal_enables_counterargument_synthesis() {
    let config = DomainConfig::for_domain(Domain::Legal);
    assert!(config.tier2_phases.counterargument_synthesis);
    assert!(!config.tier2_phases.causal_reasoning);
}

#[test]
fn test_domain_inference_medical() {
    // Note: Domain inference uses hardcoded regex in test_domain_config.rs
    // This test validates that medical.yaml patterns are comprehensive
    let texts = [
        "patient diagnosis",  // matches "patient"
        "hospital procedure", // matches "hospital"
        "disease treatment",  // matches both "disease" and "treatment"
    ];
    for text in &texts {
        assert_eq!(
            Domain::infer_from_text(text),
            Domain::Medical,
            "Failed to infer medical domain from: {}",
            text
        );
    }
}

#[test]
fn test_domain_inference_legal() {
    let texts = [
        "legal precedent",
        "court decision",
        "contract terms",
        "statute law",
        "attorney argument",
    ];
    for text in &texts {
        assert_eq!(
            Domain::infer_from_text(text),
            Domain::Legal,
            "Failed to infer legal domain from: {}",
            text
        );
    }
}

#[test]
fn test_domain_inference_finance() {
    let texts = [
        "stock market returns",
        "investment portfolio",
        "profit margin",
        "market analysis",
        "revenue forecast",
    ];
    for text in &texts {
        assert_eq!(
            Domain::infer_from_text(text),
            Domain::Finance,
            "Failed to infer finance domain from: {}",
            text
        );
    }
}

#[test]
fn test_confidence_threshold_variation() {
    let medical = ConfidenceThresholds::medical();
    let legal = ConfidenceThresholds::legal();
    let finance = ConfidenceThresholds::finance();

    // Medical should be most aggressive (lowest threshold)
    assert!(medical.hallucination_flag <= finance.hallucination_flag);
    assert!(finance.hallucination_flag < legal.hallucination_flag);
}

#[test]
fn test_weight_differences_across_domains() {
    let medical = EnsembleWeights::medical();
    let legal = EnsembleWeights::legal();
    let finance = EnsembleWeights::finance();

    // Verify each domain has unique emphasis
    // Medical: numeric high
    assert!(medical.numeric_plausibility > 1.5);

    // Legal: formal logic high
    assert!(legal.formal_logic > 1.8);

    // Finance: numeric high (similar to medical)
    assert!(finance.numeric_plausibility > 1.5);

    // But different on other dimensions
    assert!(legal.formal_logic > finance.formal_logic);
}

#[test]
fn test_tier2_phases_consistency() {
    let domains = [
        Domain::Medical,
        Domain::Legal,
        Domain::Finance,
        Domain::Science,
        Domain::Code,
    ];

    for domain in &domains {
        let config = DomainConfig::for_domain(*domain);
        // Chain of thought should always be enabled
        assert!(
            config.tier2_phases.chain_of_thought,
            "COT disabled for {:?}",
            domain
        );
        // Uncertainty should always be enabled
        assert!(
            config.tier2_phases.uncertainty_quantification,
            "UQ disabled for {:?}",
            domain
        );
    }
}

#[test]
fn test_configuration_serialization() {
    let config = DomainConfig::for_domain(Domain::Medical);
    let json = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: DomainConfig = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.domain, config.domain);
    assert_eq!(
        deserialized.ensemble_weights.numeric_plausibility,
        config.ensemble_weights.numeric_plausibility
    );
}
