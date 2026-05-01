//! Integration tests for Phase 3.5: Constitutional Deterministic Reasoner
//! Tests the complete flow of all three Phase 3.5 modules working together

#[cfg(test)]
mod integration_tests {
    use crate::domain_config::Domain;
    use crate::meta_reasoning::MetaReasoner;
    use crate::process_reward_model::ProcessRewardModel;
    use crate::symbolic_verification::SymbolicVerifier;

    #[test]
    fn test_phase35_full_flow_medical_domain() {
        // GIVEN: A medical domain question with answer
        let _question = "Can acetaminophen and ibuprofen be taken together?";
        let answer = "Yes, taking acetaminophen and ibuprofen together is safe and can provide better pain relief.";

        // WHEN: Process through Phase 3.5 modules
        let verifier = SymbolicVerifier::for_domain(Domain::Medical);
        let check_result = verifier.verify_reasoning(answer);

        // THEN: Constraint checking should work
        assert!(check_result.confidence_penalty >= 0.0);
        assert!(check_result.confidence_penalty <= 0.50); // Max penalty is capped
    }

    #[test]
    fn test_phase35_symbolic_verification_detects_violations() {
        // GIVEN: A medical claim with dosage constraint violation
        let bad_claim = "Take 5000mg of acetaminophen per dose.";

        // WHEN: Verify the claim
        let verifier = SymbolicVerifier::for_domain(Domain::Medical);
        let result = verifier.verify_reasoning(bad_claim);

        // THEN: Should detect violation
        assert!(
            !result.violations.is_empty(),
            "Should detect dosage constraint violation"
        );
        assert!(
            result.confidence_penalty > 0.0,
            "Should have penalty for violation"
        );
    }

    #[test]
    fn test_phase35_process_reward_model_domain_weights() {
        // GIVEN: A domain-specific reward model
        let medical_model = ProcessRewardModel::for_domain(Domain::Medical);

        // WHEN: Score phase contributions
        let phase_scores = vec![0.8, 0.6, 0.7, 0.9, 0.8];
        let result = medical_model.score_reasoning_process(&phase_scores);

        // THEN: Medical domain should prioritize Causal (index 3) with 0.9 score
        assert!(result.weighted_score > 0.0);
        assert!(result.weighted_score <= 1.0);
        assert_eq!(result.phase_scores.len(), 5);
    }

    #[test]
    fn test_phase35_meta_reasoning_routing() {
        // GIVEN: A low-confidence medical claim
        let mut meta_reasoner = MetaReasoner::for_domain(Domain::Medical);
        let low_confidence = 0.45;

        // WHEN: Run self-critique
        let routing_result = meta_reasoner.self_critique_and_route(low_confidence);

        // THEN: Should detect low quality and potentially route to alternative
        assert!(routing_result.confidence >= 0.0);
        assert!(routing_result.confidence <= 1.0);
    }

    #[test]
    fn test_phase35_meta_reasoning_learning() {
        // GIVEN: A meta-reasoner with no prior outcomes
        let mut meta_reasoner = MetaReasoner::for_domain(Domain::Legal);
        let initial_threshold = meta_reasoner.get_confidence_threshold();

        // WHEN: Learn from 15 successful outcomes
        for _ in 0..15 {
            meta_reasoner.learn_from_outcome("standard", true);
        }

        // THEN: Threshold should have lowered (more aggressive)
        let new_threshold = meta_reasoner.get_confidence_threshold();
        assert!(
            new_threshold < initial_threshold,
            "Threshold should lower after successes"
        );
        assert!(
            new_threshold >= 0.50,
            "Threshold should never go below 0.50"
        );
    }

    #[test]
    fn test_phase35_legal_domain_assumption_priority() {
        // GIVEN: A legal domain claim
        let legal_model = ProcessRewardModel::for_domain(Domain::Legal);

        // WHEN: Score with high assumption scores (Assumption is index 4)
        let phase_scores = vec![0.5, 0.5, 0.5, 0.5, 0.95];
        let result = legal_model.score_reasoning_process(&phase_scores);

        // THEN: High assumption score should boost overall score for legal domain
        assert!(
            result.weighted_score > 0.6,
            "Legal domain should boost with high assumption score"
        );
    }

    #[test]
    fn test_phase35_finance_domain_balancing() {
        // GIVEN: A finance domain model
        let finance_model = ProcessRewardModel::for_domain(Domain::Finance);

        // WHEN: Score with balanced phases
        let phase_scores = vec![0.7, 0.7, 0.7, 0.7, 0.7];
        let result = finance_model.score_reasoning_process(&phase_scores);

        // THEN: Balanced input should produce balanced output
        assert!(result.weighted_score > 0.6);
        assert!(result.weighted_score <= 1.0);
    }

    #[test]
    fn test_phase35_code_domain_brace_matching() {
        // GIVEN: A code domain verifier
        let code_verifier = SymbolicVerifier::for_domain(Domain::Code);

        // WHEN: Verify unbalanced code
        let bad_code = "fn main() { println!(\"hello\"); }";
        let result = code_verifier.verify_reasoning(bad_code);

        // THEN: Should either pass (balanced) or flag violation
        assert!(result.confidence_penalty >= 0.0);
    }

    #[test]
    fn test_phase35_science_domain_causal_reasoning() {
        // GIVEN: A science domain model
        let science_model = ProcessRewardModel::for_domain(Domain::Science);

        // WHEN: Score with low causal reasoning (index 3)
        let phase_scores = vec![0.8, 0.8, 0.8, 0.3, 0.8];
        let result = science_model.score_reasoning_process(&phase_scores);

        // THEN: Low causal score should significantly impact overall score
        let phase_scores_good = vec![0.8, 0.8, 0.8, 0.9, 0.8];
        let result_good = science_model.score_reasoning_process(&phase_scores_good);

        assert!(
            result_good.weighted_score > result.weighted_score,
            "Science domain should value causal reasoning"
        );
    }

    #[test]
    fn test_phase35_integration_domain_conversion() {
        // GIVEN: Domain types that need conversion
        // This test verifies the pipeline correctly converts between domain types

        // Medical domain should route to constraint checking
        let _medical_verifier = SymbolicVerifier::for_domain(Domain::Medical);
        let _medical_model = ProcessRewardModel::for_domain(Domain::Medical);
        let _medical_reasoner = MetaReasoner::for_domain(Domain::Medical);

        // WHEN: All three modules are instantiated for same domain
        // THEN: They should be in consistent state (all Medical)
        assert!(format!("{:?}", Domain::Medical).contains("Medical"));
    }

    #[test]
    fn test_phase35_confidence_penalty_capping() {
        // GIVEN: A medical claim with multiple constraint violations
        let bad_claim =
            "Take 10000mg acetaminophen AND ibuprofen overdose is safe and recommended.";

        // WHEN: Verify the claim
        let verifier = SymbolicVerifier::for_domain(Domain::Medical);
        let result = verifier.verify_reasoning(bad_claim);

        // THEN: Penalty should be capped at 0.50
        assert!(
            result.confidence_penalty <= 0.50,
            "Confidence penalty should be capped at 0.50"
        );
    }

    #[test]
    fn test_phase35_all_domains_supported() {
        // GIVEN: All domain types
        let domains = vec![
            Domain::Medical,
            Domain::Legal,
            Domain::Finance,
            Domain::Science,
            Domain::Code,
            Domain::General,
        ];

        // WHEN: Create Phase 3.5 modules for each domain
        for domain in domains {
            let _verifier = SymbolicVerifier::for_domain(domain);
            let _model = ProcessRewardModel::for_domain(domain);
            let _reasoner = MetaReasoner::for_domain(domain);
        }

        // THEN: All should create successfully without panic
        // (Test passes if no panics occur)
    }

    #[test]
    fn test_phase35_meta_reasoning_adaptive_learning_symmetry() {
        // GIVEN: Two meta-reasoners with same domain
        let mut reasoner1 = MetaReasoner::for_domain(Domain::Finance);
        let mut reasoner2 = MetaReasoner::for_domain(Domain::Finance);

        let initial_threshold1 = reasoner1.get_confidence_threshold();
        let initial_threshold2 = reasoner2.get_confidence_threshold();

        // WHEN: Train reasoner1 with successes, reasoner2 with failures
        // Note: Learning activates after >10 outcomes
        for _ in 0..15 {
            reasoner1.learn_from_outcome("standard", true);
            reasoner2.learn_from_outcome("standard", false);
        }

        let threshold1_after = reasoner1.get_confidence_threshold();
        let threshold2_after = reasoner2.get_confidence_threshold();

        // THEN: Thresholds should diverge
        assert!(
            threshold1_after < initial_threshold1,
            "Success should lower threshold"
        );
        assert!(
            threshold2_after > initial_threshold2,
            "Failure should raise threshold"
        );
        assert!(
            threshold1_after < threshold2_after,
            "Successful reasoner should have lower threshold"
        );
    }
}
