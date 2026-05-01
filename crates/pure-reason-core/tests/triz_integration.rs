//! Integration tests for TRIZ enhancements within pure-reason-core.

#[cfg(test)]
mod triz_integration_tests {
    use pure_reason_core::{
        domain_calibration::DomainCalibrator,
        meta_learner_v2::SessionMetaLearner,
        pre_verification_v2::{PreVerificationConfig, PreVerifier},
        wikipedia_corpus::WikipediaCorpus,
    };
    use std::collections::HashMap;

    #[test]
    fn test_pre_verifier_integration() {
        let pre_verifier = PreVerifier::new(PreVerificationConfig::default());

        // Test arithmetic detection
        let result = pre_verifier.pre_verify("2 + 2 = 5").unwrap();
        println!(
            "Arithmetic test: verdict={:?}, confidence={}",
            result.verdict, result.confidence
        );

        // Test complexity scoring
        let simple = pre_verifier.pre_verify("The sky is blue.").unwrap();
        let complex = pre_verifier.pre_verify("The phenomenological interpretation of quantum mechanics suggests that consciousness plays a fundamental role in wave function collapse.").unwrap();

        assert!(
            complex.complexity > simple.complexity,
            "Complex text should have higher complexity score"
        );
    }

    #[test]
    fn test_meta_learner_warmup_and_adaptation() {
        let mut learner = SessionMetaLearner::new();

        // Simulate 10 calls where kac_detector is always correct
        for _ in 0..10 {
            let mut votes = HashMap::new();
            votes.insert("kac_detector".to_string(), (true, 0.9));
            votes.insert("numeric_detector".to_string(), (false, 0.5));

            learner.update_after_verification(&votes, true); // actual verdict = true
        }

        assert_eq!(learner.call_count(), 10);
        assert!(learner.is_warmup()); // Still in warmup period (< 100)

        let stats = learner.detector_stats();
        let kac_stats = stats.get("kac_detector").unwrap();
        assert_eq!(kac_stats.0, 10); // 10 hits
        assert_eq!(kac_stats.1, 0); // 0 misses
    }

    #[test]
    fn test_domain_calibrator_general_fallback() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let mut calibrator = DomainCalibrator::new(temp_dir.path()).unwrap();

        // No domain configs, should fall back to general
        let detected = calibrator
            .detect_domain("random text without domain markers")
            .unwrap();
        assert_eq!(detected.name, "general");
        assert_eq!(detected.confidence, 1.0);

        // Test calibration (general domain uses identity transformation)
        let calibrated = detected.calibrate_ecs(0.5);
        assert!((calibrated - 0.622).abs() < 0.01); // sigmoid(0.5) ≈ 0.622
    }

    #[test]
    fn test_wikipedia_corpus_graceful_fallback() {
        // Test with non-existent path (should not panic due to lazy loading)
        let corpus = WikipediaCorpus::new("nonexistent_corpus.db").unwrap();
        assert_eq!(corpus.version(), "unknown");

        // Query should return error gracefully (not panic)
        let result = corpus.query("test", 10);
        assert!(result.is_err() || result.unwrap().is_empty());
    }

    #[test]
    fn test_end_to_end_triz_stack() {
        // Simulate a complete TRIZ-enhanced verification flow

        // 1. Pre-verification
        let pre_verifier = PreVerifier::new(PreVerificationConfig::default());
        let claim = "The patient was diagnosed with diabetes and prescribed metformin.";
        let pre_result = pre_verifier.pre_verify(claim).unwrap();

        println!(
            "Pre-verification: complexity={}, can_short_circuit={}",
            pre_result.complexity, pre_result.can_short_circuit
        );

        // 2. Domain detection
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let mut calibrator = DomainCalibrator::new(temp_dir.path()).unwrap();
        let domain = calibrator.detect_domain(claim).unwrap();

        println!(
            "Domain detected: {} (confidence={})",
            domain.name, domain.confidence
        );

        // 3. Meta-learner tracking
        let mut learner = SessionMetaLearner::new();
        let mut detector_votes = HashMap::new();
        detector_votes.insert("kac_detector".to_string(), (false, 0.8));
        detector_votes.insert("numeric_detector".to_string(), (false, 0.7));

        learner.update_after_verification(&detector_votes, false);

        println!(
            "Meta-learner: call_count={}, is_warmup={}",
            learner.call_count(),
            learner.is_warmup()
        );

        // All components should work together without panicking
        eprintln!("End-to-end integration successful");
    }
}
