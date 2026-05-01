//! # Transcendental Dialectic
//!
//! The Transcendental Dialectic is the "Logic of Illusion" — Kant's analysis of
//! how Pure Reason, when it oversteps the bounds of possible experience, inevitably
//! falls into contradictions and illusions.
//!
//! ## Structure
//!
//! - [`ideas`] — The three Transcendental Ideas (Soul, World, God)
//! - [`paralogisms`] — Four fallacious syllogisms about the Soul
//! - [`antinomies`] — Four contradictions about the World-totality
//! - [`ideal`] — The Ideal of Pure Reason (God as unconditioned ground)
//!
//! ## The Key Insight
//!
//! The Dialectic shows that when reason tries to reach the unconditioned
//! (completeness, totality, the absolute), it necessarily contradicts itself.
//! These contradictions (antinomies) prove that the Ideas of reason are only
//! **regulative** (they guide inquiry) not **constitutive** (they don't constitute objects).
//!
//! In the context of LLMs: transcendental illusions correspond to hallucinations
//! and overreach — claims beyond what the model can legitimately know.

pub mod antinomies;
pub mod coverage;
pub mod ideal;
pub mod ideas;
pub mod paralogisms;
pub mod regulative;
pub mod semantic_field;

pub use antinomies::{
    check_knowledge_vs_answer, Antinomy, AntinomyDetector, AntinomyId, AntinomyKind,
    AntinomyReport, KacEvidence, KacKind, KacReport,
};
pub use coverage::{LexicalCoverageAnalyzer, LexicalCoverageReport};
pub use ideal::{IdealOfPureReason, TheologicalIdea};
pub use ideas::{IdeaUse, TranscendentalIdea};
pub use paralogisms::{Paralogism, ParalogismDetector, ParalogismKind, ParalogismReport};
pub use regulative::{
    EpistemicCertificate, OverreachKind, RegulativeTransformation, RegulativeTransformer,
};
pub use semantic_field::{KeywordSemanticField, SemanticField};

use crate::types::Proposition;
use serde::{Deserialize, Serialize};

/// Identifies the pipeline stage that produced a [`TranscendentalIllusion`].
///
/// This typed tag replaces fragile `description.starts_with("KAC:")` string
/// matching in downstream routing and dashboard analytics.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IllusionSource {
    /// Knowledge-Answer Contradiction (KAC) from the antinomy detector.
    Kac,
    /// Entity novelty overreach — answer introduces entities absent from knowledge context.
    EntityNovelty,
    /// World Prior Capsule — answer asserts a known misconception from the static atlas.
    WorldPrior,
    /// Presupposition acceptance — answer fails to refute a false presupposition.
    PresuppositionAcceptance,
    /// Produced by the standard transcendental idea / overreach detectors.
    TranscendentalIdea,
    /// Produced by the antinomy detection subsystem (non-KAC).
    Antinomy,
    /// Produced by the paralogism detection subsystem.
    Paralogism,
}

/// A transcendental illusion is the error of treating a regulative idea
/// (a goal of inquiry) as if it were a constitutive concept (a real object).
///
/// "There is ... a natural and unavoidable dialectic of pure reason." — CPR A298/B354
///
/// In LLM terms: hallucination, confabulation, and epistemic overreach.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscendentalIllusion {
    pub id: uuid::Uuid,
    /// The idea that is being misused.
    pub idea: TranscendentalIdea,
    /// The proposition that contains the illusion.
    pub proposition: Proposition,
    /// Description of the illusion.
    pub description: String,
    /// The kind of illusion.
    pub kind: IllusionKind,
    pub severity: IllusionSeverity,
    /// The pipeline stage that produced this illusion.
    /// Use this field for routing/filtering instead of parsing `description`.
    pub source: IllusionSource,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IllusionKind {
    /// Treating an Idea as if it denoted a real object.
    HypostatizingIdea,
    /// Claiming definite knowledge of what lies beyond experience.
    EpistemicOverreach,
    /// Applying categories beyond the limits of possible experience.
    CategoryOverextension,
    /// Treating a regulative principle as constitutive.
    RegulativeConstitutive,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IllusionSeverity {
    Low,
    Medium,
    High,
    Critical,
}

// ─── IllusionDetector ────────────────────────────────────────────────────────

/// Detects transcendental illusions in propositions.
pub struct IllusionDetector;

impl IllusionDetector {
    /// Detect all transcendental illusions in a set of propositions.
    pub fn detect(propositions: &[Proposition]) -> Vec<TranscendentalIllusion> {
        let mut illusions = Vec::new();

        for prop in propositions {
            illusions.extend(Self::detect_in_proposition(prop));
        }

        illusions
    }

    fn detect_in_proposition(proposition: &Proposition) -> Vec<TranscendentalIllusion> {
        let mut illusions = Vec::new();
        let text = proposition.text.to_lowercase();

        // Detect soul-idea hypostatization
        let soul_phrases = [
            (
                "the soul is",
                IllusionKind::HypostatizingIdea,
                IllusionSeverity::High,
            ),
            (
                "i am a substance",
                IllusionKind::HypostatizingIdea,
                IllusionSeverity::High,
            ),
            (
                "the self exists",
                IllusionKind::HypostatizingIdea,
                IllusionSeverity::Medium,
            ),
            (
                "consciousness is a thing",
                IllusionKind::HypostatizingIdea,
                IllusionSeverity::High,
            ),
        ];

        for (phrase, kind, severity) in soul_phrases {
            if text.contains(phrase) {
                illusions.push(TranscendentalIllusion {
                    id: uuid::Uuid::new_v4(),
                    idea: TranscendentalIdea::Soul(ideas::PsychologicalIdea::Substantiality),
                    proposition: proposition.clone(),
                    description: format!("Possible hypostatization of the Soul-idea: '{phrase}'"),
                    kind,
                    severity,
                    source: IllusionSource::TranscendentalIdea,
                });
            }
        }

        // Detect world-idea overreach
        let world_phrases = [
            (
                "the universe is infinite",
                IllusionKind::EpistemicOverreach,
                IllusionSeverity::High,
            ),
            (
                "the world is finite",
                IllusionKind::EpistemicOverreach,
                IllusionSeverity::High,
            ),
            (
                "the universe had a beginning",
                IllusionKind::EpistemicOverreach,
                IllusionSeverity::High,
            ),
            (
                "the universe has no beginning",
                IllusionKind::EpistemicOverreach,
                IllusionSeverity::High,
            ),
            (
                "everything is determined",
                IllusionKind::CategoryOverextension,
                IllusionSeverity::Medium,
            ),
            (
                "there is no free will",
                IllusionKind::CategoryOverextension,
                IllusionSeverity::Medium,
            ),
        ];

        for (phrase, kind, severity) in world_phrases {
            if text.contains(phrase) {
                illusions.push(TranscendentalIllusion {
                    id: uuid::Uuid::new_v4(),
                    idea: TranscendentalIdea::World(ideas::CosmologicalIdea::Infinity),
                    proposition: proposition.clone(),
                    description: format!("Possible World-idea overreach: '{phrase}'"),
                    kind,
                    severity,
                    source: IllusionSource::TranscendentalIdea,
                });
            }
        }

        // Detect god-idea hypostatization
        let god_phrases = [
            (
                "god exists",
                IllusionKind::HypostatizingIdea,
                IllusionSeverity::High,
            ),
            (
                "god does not exist",
                IllusionKind::HypostatizingIdea,
                IllusionSeverity::High,
            ),
            (
                "there must be a first cause",
                IllusionKind::RegulativeConstitutive,
                IllusionSeverity::Medium,
            ),
            (
                "a necessary being exists",
                IllusionKind::HypostatizingIdea,
                IllusionSeverity::High,
            ),
            // Ontological argument formulations
            (
                "god necessarily exists",
                IllusionKind::HypostatizingIdea,
                IllusionSeverity::High,
            ),
            (
                "god exists necessarily",
                IllusionKind::HypostatizingIdea,
                IllusionSeverity::High,
            ),
            (
                "most perfect being",
                IllusionKind::HypostatizingIdea,
                IllusionSeverity::High,
            ),
            (
                "ground of all being",
                IllusionKind::RegulativeConstitutive,
                IllusionSeverity::High,
            ),
            (
                "ens realissimum",
                IllusionKind::HypostatizingIdea,
                IllusionSeverity::High,
            ),
            (
                "god is the ground",
                IllusionKind::HypostatizingIdea,
                IllusionSeverity::High,
            ),
        ];

        for (phrase, kind, severity) in god_phrases {
            if text.contains(phrase) {
                illusions.push(TranscendentalIllusion {
                    id: uuid::Uuid::new_v4(),
                    idea: TranscendentalIdea::God(ideas::TheologicalIdea::NecessaryBeing),
                    proposition: proposition.clone(),
                    description: format!("Possible God-idea hypostatization: '{phrase}'"),
                    kind,
                    severity,
                    source: IllusionSource::TranscendentalIdea,
                });
            }
        }

        illusions
    }
}

// ─── DialecticReport ─────────────────────────────────────────────────────────

/// The full dialectical analysis of a set of propositions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialecticReport {
    pub illusions: Vec<TranscendentalIllusion>,
    pub paralogisms: Vec<ParalogismReport>,
    pub antinomies: Vec<AntinomyReport>,
    /// Whether any critical illusions were found.
    pub has_critical_illusions: bool,
    /// Summary of the dialectical analysis.
    pub summary: String,
    /// KAC score from Knowledge-Answer Contradiction engine (0.0 = no conflict, 1.0 = strong conflict).
    /// `None` when no structured knowledge+answer context is available (KAC did not run).
    pub kac_score: Option<f64>,
    /// Entity novelty score (0.0 = fully grounded, 1.0 = all answer entities absent from context).
    /// `None` when no structured context is available (coverage check did not run).
    pub entity_novelty: Option<f64>,
}

impl DialecticReport {
    pub fn from_propositions(propositions: &[Proposition]) -> Self {
        let illusions = IllusionDetector::detect(propositions);
        let paralogisms = ParalogismDetector::detect(propositions);
        let antinomies = AntinomyDetector::detect(propositions);

        let has_critical = illusions
            .iter()
            .any(|i| i.severity == IllusionSeverity::Critical);

        let summary = Self::summarize(&illusions, &paralogisms, &antinomies);

        Self {
            illusions,
            paralogisms,
            antinomies,
            has_critical_illusions: has_critical,
            summary,
            kac_score: None,
            entity_novelty: None,
        }
    }

    /// Empty report for adaptive activation (TRIZ S-3): skipped layers return empty.
    pub fn empty() -> Self {
        Self {
            illusions: Vec::new(),
            paralogisms: Vec::new(),
            antinomies: Vec::new(),
            has_critical_illusions: false,
            summary: String::new(),
            kac_score: None,
            entity_novelty: None,
        }
    }

    fn summarize(
        illusions: &[TranscendentalIllusion],
        paralogisms: &[ParalogismReport],
        antinomies: &[AntinomyReport],
    ) -> String {
        let mut parts = Vec::new();

        if illusions.is_empty() && paralogisms.is_empty() && antinomies.is_empty() {
            return "No dialectical issues detected. The propositions appear to remain within the bounds of possible experience.".to_string();
        }

        if !illusions.is_empty() {
            parts.push(format!(
                "{} transcendental illusion(s) detected",
                illusions.len()
            ));
        }
        let true_paralogisms = paralogisms.iter().filter(|p| p.has_paralogisms).count();
        if true_paralogisms > 0 {
            parts.push(format!("{} paralogism(s) detected", true_paralogisms));
        }
        if !antinomies.is_empty() {
            parts.push(format!("{} antinomy/antinomies detected", antinomies.len()));
        }

        parts.join("; ") + ". Review the dialectic report for details."
    }
}

// ─── DialecticLayer ──────────────────────────────────────────────────────────

/// The Transcendental Dialectic as a faculty.
pub struct DialecticLayer;

impl crate::types::Faculty for DialecticLayer {
    type Input = Vec<Proposition>;
    type Output = DialecticReport;

    fn name(&self) -> &'static str {
        "Transcendental Dialectic"
    }

    fn apply(&self, input: Vec<Proposition>) -> crate::error::Result<DialecticReport> {
        Ok(DialecticReport::from_propositions(&input))
    }
}
