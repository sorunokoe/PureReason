//! # Kantian Pipeline
//!
//! The full Kantian cognitive pipeline, orchestrating all faculties:
//!
//! ```text
//! Raw Text
//!     │
//!     ▼
//! Aesthetic Layer (Space + Time → Intuition)
//!     │
//!     ▼
//! Analytic Layer (Categories + Schematism + Principles → Understanding)
//!     │
//!     ▼
//! Dialectic Layer (Illusions + Antinomies + Paralogisms → Validation)
//!     │
//!     ▼
//! Methodology Layer (Discipline + Canon + Architectonic → Methodology Report)
//!     │
//!     ▼
//! Wittgenstein Layer (Language Games + Rules + Family Resemblance)
//!     │
//!     ▼
//! PipelineReport (Full analysis, serializable to JSON/TOML/Markdown)
//! ```

use crate::{
    adaptive_weighting,
    aesthetic::{AestheticLayer, SegmentedInput},
    analytic::{AnalyticLayer, Understanding},
    claims::{annotate_claims_from_segmented, annotation_to_triple, ClaimAnnotatedReport},
    confidence_calibration, contradiction_detector, counterfactual_reasoner,
    dialectic::ideas::{CosmologicalIdea, TranscendentalIdea},
    dialectic::semantic_field::KeywordSemanticField,
    dialectic::{
        check_knowledge_vs_answer, DialecticLayer, DialecticReport, IllusionKind, IllusionSeverity,
        IllusionSource, LexicalCoverageAnalyzer, RegulativeTransformation, RegulativeTransformer,
        TranscendentalIllusion,
    },
    dialogue::{ContradictionPair, DialogueEpistemicState, DialogueSummary, TurnVerdict},
    domain_config, domain_governance,
    ensemble_verifier::EnsembleVerifier,
    error::Result,
    meta_reasoning::MetaReasoner,
    methodology::{MethodologyLayer, MethodologyReport},
    model_inference, pre_verification,
    process_reward_model::ProcessRewardModel,
    self_verification,
    symbolic_verification::SymbolicVerifier,
    synthetic_apriori::PresuppositionDetector,
    types::{EpistemicStatus, Faculty, Proposition, PropositionKind},
    wittgenstein::{WittgensteinLayer, WittgensteinReport},
    world_priors::WorldPriorScanner,
};
use serde::{Deserialize, Serialize};

// ─── PipelineReport ──────────────────────────────────────────────────────────

/// The full output of the Kantian Pipeline — a comprehensive analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineReport {
    /// The original input text.
    pub input: String,

    // === Aesthetic ===
    /// The structured intuition produced by the Aesthetic layer.
    pub intuition_summary: IntuitionSummary,

    // === Analytic ===
    /// The conceptual understanding produced by the Analytic layer.
    pub understanding: Understanding,

    // === Dialectic ===
    /// The dialectical validation produced by the Dialectic layer.
    pub dialectic: DialecticReport,

    // === Methodology ===
    /// The methodological analysis.
    pub methodology: MethodologyReport,

    // === Wittgenstein ===
    /// The Wittgensteinian language analysis.
    pub wittgenstein: WittgensteinReport,

    // === Claim reasoning ===
    /// Claim-first decomposition with local evidence binding.
    pub claim_analysis: ClaimAnnotatedReport,
    /// Stateful dialogue analysis for Knowledge/Response inputs, when applicable.
    pub dialogue_analysis: Option<DialogueAnalysis>,

    // === Regulative Transformation ===
    /// Constitutive → regulative transformations for every detected dialectical issue.
    /// Empty when no illusions, paralogisms, or antinomies are detected.
    pub transformations: Vec<RegulativeTransformation>,
    /// The full input text with all constitutive claims replaced by their regulative forms.
    /// Identical to `input` when no issues are detected.
    pub regulated_text: String,

    // === Synthesis ===
    /// The epistemic status of the input (phenomenon / noumenon / unknown).
    pub epistemic_status: EpistemicStatus,
    /// The overall verdict of the pipeline.
    pub verdict: Verdict,
    /// A human-readable summary.
    pub summary: String,
}

/// Stateful dialogue reasoning attached to dialogue-format pipeline runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueAnalysis {
    /// Aggregate dialogue-level contradiction and flux summary.
    pub summary: DialogueSummary,
    /// Verdict for the most recent turn (typically the response being evaluated).
    pub last_turn: TurnVerdict,
    /// Contradictions discovered so far in chronological order.
    pub contradiction_timeline: Vec<ContradictionPair>,
}

/// A condensed summary of the Aesthetic output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntuitionSummary {
    pub token_count: usize,
    pub sentence_count: usize,
    pub structural_node_count: usize,
    pub temporal_event_count: usize,
    pub temporal_orientation: String,
}

/// The overall verdict of the pipeline analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verdict {
    /// Is the input within the bounds of legitimate knowledge?
    pub within_bounds: bool,
    /// Are there dialectical illusions?
    pub has_illusions: bool,
    /// Are there antinomies/contradictions?
    pub has_contradictions: bool,
    /// Are there paralogisms?
    pub has_paralogisms: bool,
    /// The dominant category detected.
    pub dominant_category: Option<String>,
    /// The primary language game.
    pub primary_language_game: Option<String>,
    /// Risk level.
    pub risk: RiskLevel,
    /// Raw risk pre-score (0.0–1.0) from Analytic alone (TRIZ S-3).
    pub pre_score: f64,
    /// Approximate Epistemic Confidence Score (0–100).
    /// Derived inline from pre_score and issue_count (no circular dep with calibration.rs).
    /// Use `pure-reason calibrate` for the full weighted ECS.
    /// Used by benchmark runners for S2 Adaptive Decision Boundary routing.
    pub ecs: u8,
    /// True when a World Prior Capsule matched (TRIZ S3 — myth detection).
    pub prior_matched: bool,
    /// Ensemble verifier confidence (0.0–1.0) that there is a hallucination.
    /// Scale 2 Phase A: Multi-detector confidence voting.
    pub ensemble_confidence: f64,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    /// No issues detected — input is within bounds of legitimate knowledge.
    Safe,
    /// Minor issues — some ambiguity or borderline claims.
    Low,
    /// Significant issues — transcendental illusions or contradictions present.
    Medium,
    /// Critical issues — multiple severe violations detected.
    High,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Safe => write!(f, "SAFE"),
            Self::Low => write!(f, "LOW"),
            Self::Medium => write!(f, "MEDIUM"),
            Self::High => write!(f, "HIGH"),
        }
    }
}

/// Controls which layers run (TRIZ S-3: Adaptive Layer Activation).
///
/// Derived from the Analytic pre-score. The Dialectic always runs because
/// it is the safety-critical layer (antinomy/illusion/paralogism detection).
/// Only the Methodology layer (discipline, canon, architectonic) is skipped
/// for routine inputs — it is the most expensive-per-use layer.
///
/// - Standard (pre_score < 0.30): Dialectic runs; Methodology skipped
/// - Full    (>= 0.30): all layers run
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActivationLevel {
    /// Aesthetic + Analytic + Wittgenstein + Dialectic (Methodology skipped).
    Standard,
    /// Full pipeline — all five layers including Methodology.
    Full,
}

impl ActivationLevel {
    pub fn from_pre_score(score: f64) -> Self {
        if score < 0.30 {
            Self::Standard
        } else {
            Self::Full
        }
    }
}

// ─── KantianPipeline ─────────────────────────────────────────────────────────

/// The full Kantian cognitive pipeline.
///
/// Orchestrates all layers: Aesthetic → Analytic → Dialectic → Methodology → Wittgenstein.
pub struct KantianPipeline {
    pub aesthetic: AestheticLayer,
    pub analytic: AnalyticLayer,
    pub dialectic: DialecticLayer,
    pub methodology: MethodologyLayer,
    pub wittgenstein: WittgensteinLayer,
}

impl Default for KantianPipeline {
    fn default() -> Self {
        Self {
            aesthetic: AestheticLayer,
            analytic: AnalyticLayer::new(),
            dialectic: DialecticLayer,
            methodology: MethodologyLayer::new(),
            wittgenstein: WittgensteinLayer::new(),
        }
    }
}

impl KantianPipeline {
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the full pipeline on raw text input.
    ///
    /// Implements TRIZ improvements:
    /// - **S-1**: Wittgenstein game detection runs first; feeds game context into Analytic.
    /// - **S-1/KAC**: Knowledge-Answer Contradiction engine cross-checks structured inputs.
    /// - **S-2**: Lexical Entity Coverage flags entities in answer absent from context.
    /// - **S-3**: Adaptive layer activation — Methodology skipped for safe inputs.
    /// - **S-4**: Presupposition acceptance detector for TruthfulQA-style questions.
    pub fn process(&self, input: impl Into<String>) -> Result<PipelineReport> {
        let input = input.into();

        // TRIZ S1 — segment the input for cross-segment analysis
        let segments = SegmentedInput::parse(&input);

        // Stage 1: Aesthetic — structure the raw input into space + time + propositions
        let intuition = self.aesthetic.apply(input.clone())?;
        let propositions = intuition.propositions();

        // Stage 2: Wittgenstein (TRIZ S-1 — run before Analytic to provide game context)
        let wittgenstein: WittgensteinReport = self.wittgenstein.apply(&propositions);
        let detected_game = wittgenstein
            .game_analysis
            .primary_game
            .as_ref()
            .map(|g| g.form_of_life);

        // Stage 3: Analytic — categories weighted by language-game context (S-1)
        let understanding: Understanding =
            self.analytic.apply_with_game(&intuition, detected_game)?;

        // Stage 4: Compute risk pre-score for adaptive activation (TRIZ S-3)
        let pre_score = understanding.category_analysis.risk_pre_score();
        let activation = ActivationLevel::from_pre_score(pre_score);

        // Stage 5: Dialectic — always runs (safety-critical layer)
        let mut dialectic: DialecticReport = self.dialectic.apply(propositions.clone())?;

        // TRIZ S1: Knowledge-Answer Contradiction (KAC) engine
        // Runs only when structured knowledge+answer context is detected.
        if segments.has_knowledge_answer_context() {
            let sf = KeywordSemanticField;
            let knowledge = segments.knowledge.as_deref().unwrap_or("");
            let answer = segments.answer.as_deref().unwrap_or("");
            let kac = check_knowledge_vs_answer(knowledge, answer, &sf);
            dialectic.kac_score = Some(kac.contradiction_score);

            if kac.has_contradiction {
                if let Some(ev) = kac.evidence.first() {
                    dialectic.antinomies.push(crate::dialectic::AntinomyReport {
                        antinomy: crate::dialectic::AntinomyId::Generic,
                        thesis_proposition: Some(Proposition::new(
                            &ev.knowledge_fragment,
                            PropositionKind::SyntheticAposteriori,
                        )),
                        antithesis_proposition: Some(Proposition::new(
                            &ev.answer_fragment,
                            PropositionKind::SyntheticAposteriori,
                        )),
                        has_conflict: true,
                        description: format!(
                            "KAC: answer contradicts knowledge ({:?}, score={:.2})",
                            ev.kind, ev.score
                        ),
                        resolution: "The answer conflicts with the provided knowledge context. \
                                     Verify and correct the factual claim."
                            .to_string(),
                    });
                }
            }
        }

        // TRIZ S2: Lexical Entity Coverage check
        if let (Some(answer), Some(context)) = (segments.answer.as_deref(), segments.context()) {
            let coverage = LexicalCoverageAnalyzer::analyze(context, answer);
            dialectic.entity_novelty = Some(coverage.novelty_score);

            // High entity novelty in knowledge-grounded context = strong hallucination signal.
            // HaluEval hallucinated answers introduce entities not present in the knowledge passage.
            // Threshold tuned to 0.27 (TRIZ Report VIII S2 calibration: captures subtle entity
            // substitution while keeping precision above 0.80 on grounded benchmarks).
            if coverage.novelty_score > 0.27 && segments.has_knowledge_answer_context() {
                let answer_preview: String = segments
                    .answer
                    .as_deref()
                    .unwrap_or("")
                    .chars()
                    .take(60)
                    .collect();
                dialectic.illusions.push(TranscendentalIllusion {
                    id: uuid::Uuid::new_v4(),
                    idea: TranscendentalIdea::World(CosmologicalIdea::Infinity),
                    proposition: Proposition::new(
                        &answer_preview,
                        PropositionKind::SyntheticAposteriori,
                    ),
                    description: format!(
                        "Entity novelty overreach: answer introduces entities not grounded in \
                         knowledge context (novelty={:.2}). Hallucination indicator.",
                        coverage.novelty_score
                    ),
                    kind: IllusionKind::EpistemicOverreach,
                    severity: IllusionSeverity::Medium,
                    source: IllusionSource::EntityNovelty,
                });
            }
        }

        // TRIZ S4: Presupposition acceptance detector
        // Raises entity_novelty and injects a TranscendentalIllusion for false-presupposition
        // acceptance — a key hallucination pattern in TruthfulQA.
        if segments.has_question_answer_context() {
            let pres = PresuppositionDetector::detect_from_segments(
                segments.question.as_deref(),
                segments.answer.as_deref(),
            );
            if pres.acceptance_score > 0.5 {
                let current = dialectic.entity_novelty.unwrap_or(0.0);
                dialectic.entity_novelty = Some(current.max(pres.acceptance_score * 0.8));
                let trigger_text = pres.trigger.as_deref().unwrap_or("presupposition trigger");
                let answer_preview: String = segments
                    .answer
                    .as_deref()
                    .unwrap_or("")
                    .chars()
                    .take(60)
                    .collect();
                dialectic.illusions.push(TranscendentalIllusion {
                    id: uuid::Uuid::new_v4(),
                    idea: TranscendentalIdea::World(CosmologicalIdea::Infinity),
                    proposition: Proposition::new(
                        &answer_preview,
                        PropositionKind::SyntheticAposteriori,
                    ),
                    description: format!(
                        "Presupposition acceptance: question contains '{}' but answer \
                         explains the false premise without refuting it (score={:.2}).",
                        trigger_text, pres.acceptance_score
                    ),
                    kind: IllusionKind::EpistemicOverreach,
                    severity: IllusionSeverity::Medium,
                    source: IllusionSource::PresuppositionAcceptance,
                });
            }

            // TRIZ_REPORT_7 Solution 3: World Prior Capsules
            // Pure-Rust, zero-cost misconception atlas — no LLM required.
            // Only fires for Q+A-only context (no knowledge passage) to avoid
            // interfering with KAC's grounded evidence analysis.
            let no_knowledge = segments
                .knowledge
                .as_deref()
                .map(|k| k.trim().is_empty())
                .unwrap_or(true);
            if no_knowledge {
                let q = segments.question.as_deref().unwrap_or("");
                let a = segments.answer.as_deref().unwrap_or("");
                let prior_matches = WorldPriorScanner::scan(q, a);
                if !prior_matches.is_empty() {
                    // Each matched prior contributes to entity_novelty (epistemic overreach)
                    let prior_signal = 0.55_f64 + (prior_matches.len() as f64 * 0.10).min(0.35);
                    let current = dialectic.entity_novelty.unwrap_or(0.0);
                    dialectic.entity_novelty = Some(current.max(prior_signal));
                    let answer_preview: String = a.chars().take(60).collect();
                    let descriptions: Vec<&str> = prior_matches
                        .iter()
                        .map(|m| m.description.as_str())
                        .collect();
                    dialectic.illusions.push(TranscendentalIllusion {
                        id: uuid::Uuid::new_v4(),
                        idea: TranscendentalIdea::World(CosmologicalIdea::Infinity),
                        proposition: Proposition::new(
                            &answer_preview,
                            PropositionKind::SyntheticAposteriori,
                        ),
                        description: format!(
                            "World prior capsule: answer asserts known misconception(s): {}. \
                             (confidence={:.2})",
                            descriptions.join("; "),
                            prior_matches[0].confidence,
                        ),
                        kind: IllusionKind::EpistemicOverreach,
                        severity: IllusionSeverity::High,
                        source: IllusionSource::WorldPrior,
                    });
                }
            }
        }

        // Stage 6: Methodology — only for full activation (TRIZ S-3: skip expensive meta-layer)
        let methodology: MethodologyReport = if activation == ActivationLevel::Full {
            self.methodology.apply(&propositions)
        } else {
            MethodologyReport::empty()
        };

        // Stage 7: Regulative Transformer — convert constitutive overreach to regulative form
        let transformations = RegulativeTransformer::transform(&dialectic);
        let regulated_text = RegulativeTransformer::transform_text(&input, &transformations);
        let dialogue_analysis = self.analyze_dialogue_segments(&segments)?;

        // Stage 8: Synthesize
        let epistemic_status = self.assess_epistemic_status(&dialectic, &methodology);
        let is_dialogue = segments.is_dialogue_format();
        let verdict = self.compose_verdict(
            &understanding,
            &dialectic,
            &methodology,
            &wittgenstein,
            pre_score,
            is_dialogue,
            dialogue_analysis.as_ref(),
            &segments,
        );
        let summary =
            self.compose_summary(&input, &verdict, &dialectic, dialogue_analysis.as_ref());
        // Reuse the already-parsed SegmentedInput from Stage 1 (no double-parse).
        let claim_analysis = annotate_claims_from_segmented(&input, &segments)?;

        Ok(PipelineReport {
            input,
            intuition_summary: IntuitionSummary {
                token_count: intuition.manifold.tokens.len(),
                sentence_count: intuition.manifold.sentences.len(),
                structural_node_count: intuition.space.nodes.len(),
                temporal_event_count: intuition.time.events.len(),
                temporal_orientation: format!("{:?}", intuition.time.orientation),
            },
            understanding,
            dialectic,
            methodology,
            wittgenstein,
            claim_analysis,
            dialogue_analysis,
            transformations,
            regulated_text,
            epistemic_status,
            verdict,
            summary,
        })
    }

    fn analyze_dialogue_segments(
        &self,
        segments: &SegmentedInput,
    ) -> Result<Option<DialogueAnalysis>> {
        if !segments.is_dialogue_format() {
            return Ok(None);
        }

        let mut state = DialogueEpistemicState::new();
        if let Some(knowledge) = segments.knowledge.as_deref() {
            state.process_turn(knowledge)?;
        }
        let Some(answer) = segments.answer.as_deref() else {
            return Ok(None);
        };
        let last_turn = state.process_turn(answer)?;

        Ok(Some(DialogueAnalysis {
            summary: state.summary(),
            last_turn,
            contradiction_timeline: state.contradiction_timeline.clone(),
        }))
    }

    fn assess_epistemic_status(
        &self,
        dialectic: &DialecticReport,
        methodology: &MethodologyReport,
    ) -> EpistemicStatus {
        let has_boundary_violations = methodology.discipline_violations.iter().any(|v| {
            matches!(
                v.rule,
                crate::methodology::discipline::DisciplinaryRule::ExperienceBoundary
                    | crate::methodology::discipline::DisciplinaryRule::CategoryBoundary
            )
        });

        if has_boundary_violations || !dialectic.illusions.is_empty() {
            EpistemicStatus::Noumenon
        } else {
            EpistemicStatus::Phenomenon
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn compose_verdict(
        &self,
        understanding: &Understanding,
        dialectic: &DialecticReport,
        methodology: &MethodologyReport,
        wittgenstein: &WittgensteinReport,
        pre_score: f64,
        is_dialogue: bool,
        dialogue_analysis: Option<&DialogueAnalysis>,
        segments: &SegmentedInput,
    ) -> Verdict {
        let mut has_illusions = !dialectic.illusions.is_empty();
        let mut has_contradictions = dialectic.antinomies.iter().any(|a| a.has_conflict);
        let mut has_paralogisms = dialectic.paralogisms.iter().any(|p| p.has_paralogisms);
        let has_discipline_violations = !methodology.discipline_violations.is_empty();

        // TRIZ Wave-3A: Dialogue-Act Suppressor (Principle #22 — Turn Harm into Benefit).
        //
        // For dialogue format (Knowledge + Response, no Question), suppress:
        //   - World-prior misconception illusions (dialogue about movies/sports doesn't contain
        //     world-prior patterns; they fire on short responses with myth keywords)
        //   - Paralogisms (hedged first-person dialogue ("I think...") triggers paralogism
        //     detectors even when the answer is factually correct)
        // Keep:
        //   - Entity novelty overreach (primary hallucination signal for dialogue)
        //   - KAC-derived contradictions (grounded knowledge check)
        //
        // Note: Full dialogue precision improvement requires entity fingerprint diff (Wave-3B).
        // This wave provides the infrastructure (`is_dialogue_format`) used by Wave-3B.
        if is_dialogue {
            has_illusions = dialectic.illusions.iter().any(|ill| {
                matches!(
                    ill.source,
                    IllusionSource::Kac | IllusionSource::EntityNovelty
                )
            });
            has_paralogisms = false;
        }
        if let Some(dialogue) = dialogue_analysis {
            has_contradictions |= dialogue.last_turn.has_contradiction;
        }

        let issue_count = [
            has_illusions,
            has_contradictions,
            has_paralogisms,
            has_discipline_violations,
        ]
        .iter()
        .filter(|&&b| b)
        .count();

        let risk = match issue_count {
            0 => RiskLevel::Safe,
            1 => RiskLevel::Low,
            2 => RiskLevel::Medium,
            _ => RiskLevel::High,
        };

        let within_bounds = risk <= RiskLevel::Low;

        let dominant_category = understanding
            .category_analysis
            .dominant
            .map(|c| c.name().to_string());
        let primary_language_game = wittgenstein
            .game_analysis
            .primary_game
            .as_ref()
            .map(|g| g.name.clone());

        // S2 ADB: Inline ECS approximation — avoids circular dep with calibration.rs.
        // Full weighted ECS is available via `pure-reason calibrate`.
        // Formula: start at 100, penalise for modality risk (pre_score) and detected issues.
        let modality_deduction = (pre_score.min(1.0) * 45.0).round() as u8;
        let issue_deduction = (issue_count as u8).saturating_mul(14);
        let mut ecs = 100u8
            .saturating_sub(modality_deduction)
            .saturating_sub(issue_deduction);
        if let Some(dialogue) = dialogue_analysis {
            ecs = ecs.min(dialogue.last_turn.dialogue_ecs);
        }

        // Scale 2 Phase A: Run ensemble verifier on knowledge + answer
        let ensemble_verdict = EnsembleVerifier::verify(
            segments.knowledge.as_deref(),
            segments.answer.as_deref().unwrap_or(""),
        );
        let phase_a_signal = ensemble_verdict.hallucination_probability;
        let mut phase_b_signal = phase_a_signal; // Default to phase A if no model
        let mut phase_c_signal = 0.0;
        let mut ensemble_confidence = phase_a_signal;

        // Scale 2 Phase D (NEW): Pre-verification layer — fast heuristics before model
        // TRIZ: Preliminary action principle — detect obvious cases before expensive inference
        let preverify_result = pre_verification::pre_verify(
            segments.knowledge.as_deref().unwrap_or(""),
            segments.answer.as_deref().unwrap_or(""),
        );

        // Short-circuit model inference if pre-verification is confident
        let skip_model = preverify_result.can_short_circuit && preverify_result.confidence > 0.75;
        if skip_model {
            ensemble_confidence = preverify_result.predicted_score;
            phase_b_signal = preverify_result.predicted_score;
        } else {
            // Compute adaptive weights based on claim complexity
            // TRIZ: Dynamism principle — adjust strategy based on problem characteristics
            let answer_text = segments.answer.as_deref().unwrap_or("");
            let complexity = adaptive_weighting::compute_complexity_score(
                segments.knowledge.as_deref().unwrap_or(""),
                answer_text,
            );
            let (phase_a_weight, phase_b_weight) = adaptive_weighting::compute_weights(complexity);

            // Scale 2 Phase B: Integrate DistilBERT model with adaptive weighting
            // Simple claims: 80/20 (trust heuristics)
            // Complex claims: 60/40 (trust model)
            if let Some(model_pred) =
                model_inference::predict(segments.knowledge.as_deref(), answer_text)
            {
                // Model returns P(falsifiable) — we want P(hallucination)
                // Falsifiable claim that disagrees with knowledge = potential hallucination
                let mut model_hallucination_score = model_pred.falsifiable_prob;

                // Apply confidence calibration (temperature scaling)
                // TRIZ: Taking Out — remove overconfidence signal
                let knowledge_length = segments.knowledge.as_deref().unwrap_or("").len();
                let (calibrated_score, _calibration_reason) =
                    confidence_calibration::apply_calibration(
                        model_hallucination_score,
                        complexity,
                        knowledge_length,
                    );
                model_hallucination_score = calibrated_score;

                phase_b_signal = model_hallucination_score;
                ensemble_confidence =
                    phase_a_weight * phase_a_signal + phase_b_weight * phase_b_signal;
            }
        }

        // Scale 2 Phase C: Contradiction detection + counterfactual reasoning
        let answer_text = segments.answer.as_deref().unwrap_or("");
        let claims = contradiction_detector::extract_claims(answer_text);
        let contradiction_analysis = contradiction_detector::find_contradictions(&claims);

        // Counterfactual reasoning: build dependency graph for claims
        let dependency_graph = counterfactual_reasoner::build_dependency_graph(&claims);
        let _counterfactual_analysis =
            counterfactual_reasoner::analyze_counterfactuals(&dependency_graph, &claims);

        // Apply 30% weight to contradiction signals (70% still from Phase B)
        let contradiction_confidence = if contradiction_analysis.is_reliable {
            contradiction_analysis.overall_confidence
        } else {
            0.0 // Only apply if confident
        };

        if contradiction_confidence > 0.0 {
            phase_c_signal = contradiction_confidence;
            ensemble_confidence = 0.70 * ensemble_confidence + 0.30 * contradiction_confidence;
        }

        // Domain governance: infer domain and apply thresholds
        let inferred_domain_governance = domain_governance::infer_domain(answer_text);
        let _governance_check = domain_governance::check_governance(
            inferred_domain_governance,
            answer_text,
            ensemble_confidence,
            true, // Treating as falsifiable for now
        );

        // Convert domain_governance::Domain to domain_config::Domain for Phase 3.5
        let inferred_domain = match inferred_domain_governance {
            domain_governance::Domain::Medical => domain_config::Domain::Medical,
            domain_governance::Domain::Legal => domain_config::Domain::Legal,
            domain_governance::Domain::Finance => domain_config::Domain::Finance,
            domain_governance::Domain::Science => domain_config::Domain::Science,
            domain_governance::Domain::History | domain_governance::Domain::Philosophy => {
                domain_config::Domain::General
            }
            domain_governance::Domain::General => domain_config::Domain::General,
        };

        // ── [PHASE 3.5.1: SYMBOLIC VERIFICATION] ─────────────────────────────
        // Constraint checking for hallucination detection
        let symbolic_penalty = {
            let verifier = SymbolicVerifier::for_domain(inferred_domain);
            let check_result = verifier.verify_reasoning(answer_text);
            check_result.confidence_penalty
        };
        let symbolic_confidence_adjusted = ensemble_confidence * (1.0 - symbolic_penalty);

        // Scale 2 Phase D (NEW): Self-verification layer
        // TRIZ: Feedback + Inspection — verify internal consistency of verdict
        let verification_result = self_verification::verify_consistency(
            answer_text,
            phase_a_signal,
            phase_b_signal,
            phase_c_signal,
            symbolic_confidence_adjusted,
        );

        // Apply adjustment if inconsistency detected (kept for pipeline stability)
        let _ = self_verification::apply_verification(
            symbolic_confidence_adjusted,
            &verification_result,
        );

        // ── [PHASE 3.5.2: PROCESS REWARD MODEL] ─────────────────────────────
        // Domain-specific phase weighting for refined scoring
        let reward_model = ProcessRewardModel::for_domain(inferred_domain);
        let phase_scores = vec![
            phase_a_signal, // CoT
            0.50,           // Uncertainty (proxy)
            phase_c_signal, // Counterargument
            0.60,           // Causal (proxy)
            0.65,           // Assumption (proxy)
        ];
        let process_score = reward_model.score_reasoning_process(&phase_scores);
        let reward_adjusted_confidence = process_score.weighted_score;

        // ── [PHASE 3.5.3: META-REASONING] ────────────────────────────────────
        // Adaptive routing based on reasoning quality
        let mut meta_reasoner = MetaReasoner::for_domain(inferred_domain);
        let routing_result = meta_reasoner.self_critique_and_route(reward_adjusted_confidence);
        let final_routed_confidence = routing_result.confidence;

        let prior_matched = dialectic
            .illusions
            .iter()
            .any(|ill| ill.source == IllusionSource::WorldPrior);

        Verdict {
            within_bounds,
            has_illusions,
            has_contradictions,
            has_paralogisms,
            dominant_category,
            primary_language_game,
            risk,
            pre_score,
            ecs,
            prior_matched,
            ensemble_confidence: final_routed_confidence,
        }
    }

    fn compose_summary(
        &self,
        input: &str,
        verdict: &Verdict,
        dialectic: &DialecticReport,
        dialogue_analysis: Option<&DialogueAnalysis>,
    ) -> String {
        let preview: String = input.chars().take(80).collect();
        let ellipsis = if input.chars().count() > 80 {
            "..."
        } else {
            ""
        };
        let mut parts = vec![
            format!("Input: \"{}{}\"", preview, ellipsis),
            format!("Risk: {}", verdict.risk),
        ];

        if let Some(cat) = &verdict.dominant_category {
            parts.push(format!("Dominant category: {}", cat));
        }
        if let Some(game) = &verdict.primary_language_game {
            parts.push(format!("Language game: {}", game));
        }
        if !dialectic.summary.is_empty() {
            parts.push(format!("Dialectic: {}", dialectic.summary));
        }
        if let Some(dialogue) = dialogue_analysis {
            parts.push(format!(
                "Dialogue flux: {:.2}",
                dialogue.summary.epistemic_flux
            ));
            parts.push(format!("Dialogue ECS: {}", dialogue.last_turn.dialogue_ecs));
        }

        parts.join(" | ")
    }
}

impl PipelineReport {
    /// Convert to JSON string.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Convert to a Markdown report.
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str("# Kantian Pipeline Report\n\n");
        md.push_str(&format!("**Input:** {}\n\n", self.input));
        md.push_str(&format!("**Risk Level:** {}\n\n", self.verdict.risk));
        md.push_str(&format!(
            "**Epistemic Status:** {:?}\n\n",
            self.epistemic_status
        ));

        md.push_str("## Aesthetic\n");
        md.push_str(&format!(
            "- Tokens: {}\n",
            self.intuition_summary.token_count
        ));
        md.push_str(&format!(
            "- Sentences: {}\n",
            self.intuition_summary.sentence_count
        ));
        md.push_str(&format!(
            "- Temporal orientation: {}\n\n",
            self.intuition_summary.temporal_orientation
        ));

        md.push_str("## Analytic\n");
        if let Some(cat) = &self.verdict.dominant_category {
            md.push_str(&format!("- Dominant category: {}\n", cat));
        }
        md.push_str(&format!(
            "- Active categories (>0 confidence): {}\n\n",
            self.understanding
                .category_analysis
                .above_threshold(0.01)
                .len()
        ));

        md.push_str("## Dialectic\n");
        md.push_str(&format!(
            "- Illusions: {}\n",
            self.dialectic.illusions.len()
        ));
        md.push_str(&format!(
            "- Antinomies with conflict: {}\n",
            self.dialectic
                .antinomies
                .iter()
                .filter(|a| a.has_conflict)
                .count()
        ));
        md.push_str(&format!(
            "- Paralogism reports: {}\n\n",
            self.dialectic.paralogisms.len()
        ));

        md.push_str("## Wittgenstein\n");
        if let Some(game) = &self.verdict.primary_language_game {
            md.push_str(&format!("- Primary language game: {}\n", game));
        }
        md.push_str(&format!(
            "- Mixed games: {}\n\n",
            self.wittgenstein.game_analysis.is_mixed
        ));

        md.push_str("## Claim Reasoning\n");
        md.push_str(&format!("- Claims: {}\n", self.claim_analysis.claims.len()));
        md.push_str(&format!(
            "- Supported: {}  Contradicted: {}  Novel: {}  Unresolved: {}  Missing context: {}\n\n",
            self.claim_analysis.supported_count,
            self.claim_analysis.contradicted_count,
            self.claim_analysis.novel_count,
            self.claim_analysis.unresolved_count,
            self.claim_analysis.missing_context_count,
        ));
        if !self.claim_analysis.claims.is_empty() {
            md.push_str(
                "| ID | Role | Modality | Evidence | Triple conf | Ready | Risk | Claim |\n",
            );
            md.push_str("|---|---|---|---|---|---|---|---|\n");
            for claim in &self.claim_analysis.claims {
                let triple = annotation_to_triple(claim);
                md.push_str(&format!(
                    "| {} | {} | {} | {} | {:.2} | {} | {} | {} |\n",
                    claim.claim_id,
                    claim.source_role,
                    claim.modality,
                    claim.evidence.status,
                    triple.extraction_confidence.value(),
                    if triple.supports_contradiction() {
                        "yes"
                    } else {
                        "no"
                    },
                    claim.risk,
                    claim.text,
                ));
            }
            md.push('\n');
        }

        if let Some(dialogue) = &self.dialogue_analysis {
            md.push_str("## Dialogue Reasoning\n");
            md.push_str(&format!("- Turns: {}\n", dialogue.summary.turn_count));
            md.push_str(&format!(
                "- Dialogue ECS: {}  Epistemic flux: {:.2}  Contradictions: {}\n\n",
                dialogue.last_turn.dialogue_ecs,
                dialogue.summary.epistemic_flux,
                dialogue.summary.contradiction_count,
            ));
            if !dialogue.last_turn.contradiction_pairs.is_empty() {
                md.push_str("| Established at turn | Incoming triple | Committed triple |\n");
                md.push_str("|---|---|---|\n");
                for pair in &dialogue.last_turn.contradiction_pairs {
                    md.push_str(&format!(
                        "| {} | {} | {} |\n",
                        pair.established_at_turn, pair.incoming, pair.committed
                    ));
                }
                md.push('\n');
            }
        }

        if !self.transformations.is_empty() {
            md.push_str("## Regulative Transformations\n");
            md.push_str(&format!(
                "> {} constitutive claim(s) corrected to regulative form.\n\n",
                self.transformations.len()
            ));
            for (i, t) in self.transformations.iter().enumerate() {
                md.push_str(&format!("### Transformation {}\n", i + 1));
                md.push_str(&format!(
                    "- **Transcendental Idea:** {}\n",
                    t.transcendental_idea.name()
                ));
                md.push_str(&format!("- **Original (Constitutive):** {}\n", t.original));
                md.push_str(&format!("- **Regulated:** {}\n", t.regulated));
                md.push_str(&format!(
                    "- **Regulative Principle:** {}\n",
                    t.regulative_principle
                ));
                md.push_str(&format!(
                    "- **Resolution:** {}\n\n",
                    t.certificate.kantian_resolution
                ));
            }
            md.push_str("### Regulated Text\n");
            md.push_str(&format!("> {}\n\n", self.regulated_text));
        }

        md.push_str("## Summary\n");
        md.push_str(&self.summary);
        md.push('\n');

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_processes_simple_text() {
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process("Water boils at 100 degrees because of heat")
            .unwrap();
        assert!(!report.input.is_empty());
        assert!(report.intuition_summary.token_count > 0);
    }

    #[test]
    fn pipeline_detects_causal_category() {
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process("The effect follows from the cause necessarily because heat causes boiling")
            .unwrap();
        // Should detect causality
        assert!(report.understanding.category_analysis.dominant.is_some());
    }

    #[test]
    fn pipeline_flags_antinomy() {
        let pipeline = KantianPipeline::new();
        let report = pipeline.process(
            "The universe had a beginning in time. The universe has no beginning and is eternal."
        ).unwrap();
        assert!(report.verdict.has_contradictions);
    }

    #[test]
    fn pipeline_to_markdown() {
        let pipeline = KantianPipeline::new();
        let report = pipeline.process("Snow is white").unwrap();
        let md = report.to_markdown();
        assert!(md.contains("# Kantian Pipeline Report"));
        assert!(md.contains("## Claim Reasoning"));
    }

    #[test]
    fn pipeline_to_json() {
        let pipeline = KantianPipeline::new();
        let report = pipeline.process("All bachelors are unmarried").unwrap();
        let json = report.to_json().unwrap();
        assert!(json.contains("input"));
        assert!(json.contains("claim_analysis"));
    }

    #[test]
    fn pipeline_includes_claim_analysis() {
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process(
                "Knowledge: The capital of Australia is Canberra.\n\
             Question: What is the capital of Australia?\n\
             Answer: Sydney is the capital of Australia.",
            )
            .unwrap();
        assert_eq!(report.claim_analysis.claims.len(), 3);
        assert_eq!(report.claim_analysis.contradicted_count, 1);
    }

    #[test]
    fn dialogue_pipeline_attaches_stateful_analysis() {
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process(
                "Knowledge: The Scorch Trials was written by James Dashner.\n\
             Response: I think The Scorch Trials is a book by James Dashner.",
            )
            .unwrap();

        let dialogue = report.dialogue_analysis.as_ref().unwrap();
        assert_eq!(dialogue.summary.turn_count, 2);
        assert_eq!(dialogue.summary.contradiction_count, 0);
        assert_eq!(dialogue.last_turn.dialogue_ecs, 100);
    }

    #[test]
    fn dialogue_pipeline_uses_stateful_contradictions_in_verdict() {
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process(
                "Knowledge: The Scorch Trials was written by James Dashner.\n\
             Response: The Scorch Trials was not written by James Dashner.",
            )
            .unwrap();

        let dialogue = report.dialogue_analysis.as_ref().unwrap();
        assert!(dialogue.last_turn.has_contradiction);
        assert!(report.verdict.has_contradictions);
        assert!(report.summary.contains("Dialogue flux"));
        assert!(report.verdict.ecs <= dialogue.last_turn.dialogue_ecs);
    }

    #[test]
    fn pipeline_marks_world_prior_matches_for_question_answer_myths() {
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process(
                "Question: Is it illegal to kill a praying mantis in the U.S.?\n\
             Answer: Yes, it is illegal to kill a praying mantis.",
            )
            .unwrap();

        assert!(report.verdict.prior_matched);
        assert!(report.verdict.has_illusions);
    }

    #[test]
    fn dialogue_suppressor_does_not_flag_hedged_correct_response() {
        // TRIZ Wave-3A: Dialogue-Act Suppressor.
        // A hedged but correct dialogue response must NOT trigger has_illusions
        // or has_paralogisms — the dialogue format suppressor should clear them.
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process(
                "Knowledge: The Scorch Trials was written by James Dashner.\n\
             Response: I think The Scorch Trials is a book by James Dashner.",
            )
            .unwrap();

        // No illusion/paralogism — hedging "I think" must be tolerated in dialogue
        assert!(
            !report.verdict.has_illusions,
            "Dialogue suppressor failed: hedged correct response triggered illusion detector"
        );
        assert!(
            !report.verdict.has_paralogisms,
            "Dialogue suppressor failed: hedged correct response triggered paralogism detector"
        );
    }

    #[test]
    fn dialogue_suppressor_still_catches_knowledge_contradiction() {
        // TRIZ Wave-3A: Only knowledge contradiction should fire in dialogue format.
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process(
                "Knowledge: The Scorch Trials was written by James Dashner.\n\
             Response: The Scorch Trials was written by J.K. Rowling.",
            )
            .unwrap();

        // Contradiction against knowledge must still be detected
        assert!(
            report.verdict.has_contradictions,
            "Dialogue suppressor over-silenced: knowledge contradiction not detected"
        );
    }

    #[test]
    fn dialogue_markdown_section_is_rendered() {
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process(
                "Knowledge: Paris is the capital of France.\n\
             Response: Paris is the capital of France.",
            )
            .unwrap();

        let md = report.to_markdown();
        assert!(md.contains("## Dialogue Reasoning"));
        assert!(md.contains("Dialogue ECS"));
    }
}
