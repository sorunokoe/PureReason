//! # PureReason Core
//!
//! A Rust implementation of Kant's *Critique of Pure Reason* as a structured
//! reasoning and validation framework, with Wittgensteinian language tooling.
//!
//! ## Architecture
//!
//! The library mirrors Kant's four main divisions:
//!
//! - [`aesthetic`] — Transcendental Aesthetic: Space & Time as forms of intuition
//! - [`analytic`] — Transcendental Analytic: Categories, Schematism, Principles
//! - [`dialectic`] — Transcendental Dialectic: Ideas, Antinomies, Paralogisms
//! - [`methodology`] — Transcendental Methodology: Discipline, Canon, Architectonic
//! - [`wittgenstein`] — Wittgensteinian practical layer: Language games, Rule-following
//! - [`pipeline`] — The full Kantian cognitive pipeline orchestrating all layers

// ── [TIER 1 — KERNEL] ─────────────────────────────────────────────────────────
// Deterministic core: stable API, zero breaking changes without a major version
// bump, no LLM calls, microsecond hot-path.
pub mod adaptive_weighting; // Scale 2 Phase D: Adaptive Phase A/B blending based on complexity
pub mod aesthetic; // Transcendental Aesthetic (space/time intuitions)
pub mod analytic; // Transcendental Analytic (categories, schematism, principles)
pub mod api_gateway; // Scale 3 Tier 1: Rate limiting, health checks, request routing
pub mod assumption_validation; // Scale 3 Tier 2 Phase 5: Extract and validate implicit premises
pub mod batch_processor; // Scale 3 Tier 1: Batch processing (10–100× throughput)
pub mod causal_reasoning; // Scale 3 Tier 2 Phase 4: Distinguish correlation from causation
pub mod chain_of_thought; // Scale 3 Tier 2 Phase 1: Explicit reasoning step extraction and validation
pub mod claims; // Claim IR compiler + evidence-class ledger
pub mod confidence_calibration; // Scale 2 Phase D: Temperature scaling for model confidence
pub mod confidence_thresholding; // Quick Win #3: Confidence threshold tuning per benchmark
pub mod contradiction_detector; // Scale 2 Phase C: Logical contradiction detection
pub mod counterargument_synthesis; // Scale 3 Tier 2 Phase 3: Find and reconcile opposing arguments
pub mod counterfactual_reasoner; // Scale 2 Phase C: Counterfactual reasoning + dependency tracing
pub mod dialectic; // Transcendental Dialectic (illusions, antinomies, paralogisms)
pub mod domain_calibration; // TRIZ P3, P4: Per-domain ensemble weights and ECS calibration
pub mod domain_config; // Quick Win #1: Domain-specific hyperparameter configuration
pub mod domain_governance; // Scale 2 Phase C: Domain-specific governance policies
pub mod domain_prompts; // Quick Win #2: Domain-specific prompt templates
pub mod domain_rules; // Scale 3 Tier 1: Domain-specific validation rules
pub mod ensemble_verifier; // Scale 2 Phase A: Multi-detector confidence voting
pub mod error; // Crate-wide error type
pub mod error_analyzer; // Medium Win #7: Top failure pattern analysis & fixes
pub mod math_solver; // Medium Win #6: Specialized mathematical computation validation
pub mod meta_learner; // Home Run #9: Meta-learning from benchmark failures
pub mod meta_learner_v2; // TRIZ P13, P2: Session-scoped adaptive learning
pub mod methodology; // Transcendental Methodology (discipline, canon, architectonic)
pub mod model_inference; // Scale 2 Phase B: DistilBERT model inference
pub mod multi_hop_reasoner; // Medium Win #5: Extended reasoning chains (3-5 hops)
pub mod numeric_plausibility; // Physical/mathematical constant plausibility scanner
pub mod performance_monitor; // Scale 3 Tier 1: Latency tracking and throughput metrics
pub mod phase_optimizer; // Quick Win #4: Per-phase enable/disable optimization
pub mod pipeline; // Full Kantian cognitive pipeline
pub mod pre_verification; // Scale 2 Phase D: Fast pre-verification before model inference
pub mod pre_verification_v2; // TRIZ P10, P25: Enhanced pre-gate with arithmetic, blacklist, complexity routing
pub mod self_verification; // Scale 2 Phase D: Verdict self-consistency verification
pub mod semantic_fallback; // TRIZ P1: Embedding-based hallucination detection
pub mod types; // Shared primitive types
pub mod uncertainty_calibration; // Medium Win #8: Domain-specific confidence interval fitting
pub mod uncertainty_quantification; // Scale 3 Tier 2 Phase 2: Confidence intervals and uncertainty propagation
pub mod wikipedia_corpus; // TRIZ P40: 6M Wikipedia articles with BM25 search
pub mod wittgenstein; // Wittgensteinian language-game layer
pub mod world_priors; // Misconception atlas + BM25 soft-prior matcher

// ── [TIER 2 — PHASE 3.5: NEXT-GEN ARCHITECTURE] ────────────────────────────
// Constitutional Deterministic Reasoning: Neurosymbolic + Process Rewards + Meta-Reasoning
pub mod meta_reasoning;
pub mod process_reward_model; // Phase 3.5.2: Domain-specific step-level reward scoring
pub mod symbolic_verification; // Phase 3.5.1: Constraint checking via symbolic layer // Phase 3.5.3: Self-critique and dynamic phase routing

// ── [TIER 2 — RUNTIME] ────────────────────────────────────────────────────────
// Service orchestration: may evolve between minor versions; depends on Tier 1.
pub mod apperception; // Unity of apperception (context synthesis)
pub mod auth; // API key registry and webhook host validation
pub mod auto_calibration; // Phase 3: Auto-calibration per domain (30m vs 4h manual)
pub mod benchmark_integration; // Unified benchmarking suite integration (BIG-Bench, GSM8K, etc)
pub mod benchmark_publisher; // Phase 3: Benchmark results publishing + competitive analysis
pub mod calibration; // Pipeline calibration and ECS scoring
pub mod certificate; // Validation certificates
pub mod claims_validation; // Phase 3: Validate all 5 best-in-class claims empirically
pub mod competitive_analysis; // Benchmark domination & market positioning analysis
pub mod compliance; // Domain compliance checks
pub mod domain; // Domain-aware routing
pub mod feedback; // Human feedback integration
pub mod human_feedback; // Home Run #10: Expert validation and active learning
pub mod parallel_benchmark; // Phase 3: Async/parallel benchmarking (44K+ tasks in 2-3 hours)
pub mod rewriter; // Regulative text rewriter
pub mod session; // Session management
pub mod specialized_benchmarks; // GSM8K, HumanEval, MMLU-Pro, ARC, DROP benchmarks
pub mod structured_validator; // Domain-specialised validators (legal, medical, financial)
pub mod temporal_coherence; // Temporal coherence layer
pub mod trust_ops; // Trust operations store and SLA monitor
pub mod unity; // Unity of consciousness layer
pub mod world_model; // Dynamic world model
pub mod world_schema; // World knowledge schema loader

// ── [TIER 3 — EXPERIMENTAL] ───────────────────────────────────────────────────
// New capabilities under active development; API may change between any version.
pub mod dialogue; // Multi-turn dialogue management
pub mod imagination; // Productive imagination (schema composition)
pub mod multiagent; // Multi-agent orchestration
pub mod streaming; // Streaming response support
pub mod synthetic_apriori; // Synthetic a priori claim verification

pub use auth::{
    ensure_auth_configuration, is_disallowed_webhook_host, is_loopback_bind, ApiKeyRegistry,
    ApiPrincipal,
};
pub use calibration::{CalibrationResult, EcsBand, PipelineCalibration, ScoreBreakdown};
pub use claims::{
    annotate_claims, annotate_claims_from_segmented, annotation_to_triple, claim_type_is_factual,
    classify_claim_type, find_triple_contradictions, report_to_triples, route_for_type,
    route_summary, ClaimAnnotatedReport, ClaimAnnotation, ClaimEvidenceBinding,
    ClaimEvidenceStatus, ClaimModality, ClaimPolarity, ClaimRole, ClaimRoute, ClaimTriple,
    ClaimTripleProvenance, ClaimType, ClaimVerifier,
};
pub use ensemble_verifier::{
    DetectorVote, EnsembleVerdict, EnsembleVerifier, FormalLogicChecker, NoveltyDetector,
    NumericDomainDetector, SemanticDriftDetector,
};
pub use error::{PureReasonError, Result};
pub use meta_reasoning::{MetaReasoner, MetaRoutingResult, QualityAnalysis};
pub use model_inference::{predict, ModelPrediction};
pub use numeric_plausibility::{NumericIssue, NumericPlausibilityScanner};
pub use process_reward_model::{ProcessRewardModel, ProcessScore};
pub use rewriter::{DomainRewriter, RewriteDomain, RewriteResult};
pub use symbolic_verification::{
    Constraint, ConstraintSeverity, ConstraintViolation, SymbolicVerifier, VerificationResult,
};
pub use temporal_coherence::{TemporalCoherenceLayer, TemporalIssue, TemporalIssueKind};
pub use trust_ops::{
    default_ops_dir, evaluate_report, AuditEvent, AuditEventKind, OpsExportBundle, OpsHistoryPoint,
    OpsOverview, PolicyAction, PolicyDecision, ReviewItem, ReviewOutcome, ReviewResolution,
    ReviewStatus, ReviewUpdate, RiskyClaimSummary, TrustCounts, TrustEvaluation, TrustOpsStore,
    TrustReceipt, TrustRole,
};
pub use types::*;

// Re-export knowledge base for downstream crates.
pub use pure_reason_kb as kb;

#[cfg(test)]
mod integration_tests_phase35;
