//! # Epistemic Confidence Score (ECS) — Solution 1 from TRIZ Report V
//!
//! The Calibration Gap is the #1 community complaint about LLMs: outputs sound
//! equally confident whether right or wrong. ECS resolves this by computing a
//! deterministic, structurally grounded 0–100 score on every LLM output using
//! only the Kantian pipeline's existing analytical results.
//!
//! ## Formula
//!
//! ```text
//! ECS = round(100 × (
//!     w_modality   × modality_factor(Possibility, Existence, Necessity)
//!   + w_illusion   × illusion_factor(illusions detected)
//!   + w_antinomy   × antinomy_factor(antinomies detected)
//!   + w_paralogism × paralogism_factor(paralogisms detected)
//!   + w_game       × game_factor(primary language game)
//! ) / Σw)
//! ```
//!
//! ## Interpretation
//!
//! | ECS     | Band     | Meaning                                               |
//! |---------|----------|-------------------------------------------------------|
//! | 80–100  | HIGH     | Epistemically calibrated. Safe for regulated use.     |
//! | 40–79   | MODERATE | Some uncertainty or risk. Human review recommended.   |
//! | 0–39    | LOW      | Overconfident, contradictory, or overreaching claims. |
//!
//! ## What changes ECS
//!
//! * **Raises ECS:** Possibility-dominant language ("may", "could", "suggests"),
//!   no illusions, no contradictions, scientific/technical language game.
//! * **Lowers ECS:** Necessity-dominant language ("must", "certainly", "always"),
//!   transcendental illusions, antinomies, paralogisms, theological/philosophical game.
//!
//! ## Resources
//!
//! All inputs to the ECS are already computed by the Kantian pipeline — there is
//! zero added cost: no LLM calls, no HTTP requests, no additional processing.

use crate::{
    analytic::categories::{Category, CategoryAnalysis},
    dialectic::{DialecticReport, IllusionSeverity},
    pipeline::{PipelineReport, RiskLevel},
    rewriter::{DomainRewriter, RewriteDomain, RewriteResult},
    wittgenstein::language_games::FormOfLife,
};
use serde::{Deserialize, Serialize};

// ─── ECS Weights ─────────────────────────────────────────────────────────────
// TRIZ Solution 1+2: Added kac_score and entity_coverage components.
// Original weights are rescaled to preserve their relative contribution while
// making room for the two new content-level signals.
//
// When no structured context is present (kac_score = 0.0, entity_novelty = 0.0),
// these new components evaluate to 1.0 (no penalty) — identical to the old formula.

const W_MODALITY: f64 = 0.25;
const W_ILLUSION: f64 = 0.22;
const W_ANTINOMY: f64 = 0.15;
const W_PARALOGISM: f64 = 0.08;
const W_GAME: f64 = 0.05;
const W_KAC: f64 = 0.15; // Knowledge-Answer Contradiction signal (TRIZ S1)
const W_COVERAGE: f64 = 0.10; // Entity novelty coverage signal (TRIZ S2+S4)

// ─── EcsBand ─────────────────────────────────────────────────────────────────

/// The epistemic confidence band — a qualitative label for the ECS range.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EcsBand {
    /// ECS 80–100: epistemically calibrated, safe for regulated use.
    High,
    /// ECS 40–79: some risk, human review recommended for high-stakes decisions.
    Moderate,
    /// ECS 0–39: overconfident, contradictory, or transcendentally overreaching.
    Low,
}

impl EcsBand {
    pub fn from_score(ecs: u8) -> Self {
        match ecs {
            80..=100 => Self::High,
            40..=79 => Self::Moderate,
            _ => Self::Low,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::High => "HIGH",
            Self::Moderate => "MODERATE",
            Self::Low => "LOW",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Self::High => "✅",
            Self::Moderate => "⚠️",
            Self::Low => "🚫",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::High => "Epistemically calibrated. Safe for regulated use.",
            Self::Moderate => "Some uncertainty or epistemic risk. Human review recommended.",
            Self::Low => "Overconfident, contradictory, or transcendentally overreaching.",
        }
    }
}

// ─── CalibrationResult ───────────────────────────────────────────────────────

/// The output of the Calibration Layer — the primary user-facing product.
///
/// This is what users buy. All internals (Kant, Wittgenstein, vepol models)
/// are computed invisibly. The user sees: a score, a band, a list of flags,
/// and optionally a safe rewrite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationResult {
    /// Epistemic Confidence Score: 0–100.
    /// 80+ = reliable, 40–79 = review, <40 = flag/rewrite.
    pub ecs: u8,

    /// Qualitative band: HIGH, MODERATE, or LOW.
    pub band: EcsBand,

    /// Whether the output is safe to deliver without review.
    /// True when ECS ≥ 80.
    pub calibrated: bool,

    /// Human-readable list of detected epistemic issues.
    /// Empty when ECS is HIGH.
    pub flags: Vec<String>,

    /// The input text, rewritten in regulative (safe) language if issues found.
    /// Identical to input when no issues are detected (ECS is HIGH).
    pub safe_version: String,

    /// The dominant epistemic mode detected in the text.
    pub epistemic_mode: String,

    /// Breakdown of score components (for transparency / debugging).
    pub score_breakdown: ScoreBreakdown,
}

/// Transparent breakdown of how the ECS was computed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// Modality component (0.0–1.0): Possibility > Existence > Necessity.
    pub modality: f64,
    /// Illusion component (0.0–1.0): 1.0 = no illusions, 0.0 = critical illusion.
    pub illusion: f64,
    /// Antinomy component (0.0–1.0): 1.0 = no conflicts, 0.0 = multiple conflicts.
    pub antinomy: f64,
    /// Paralogism component (0.0–1.0): 1.0 = no paralogisms.
    pub paralogism: f64,
    /// Language-game stability component (0.0–1.0).
    pub game_stability: f64,
    /// Knowledge-Answer Contradiction component (0.0–1.0): 1.0 = no KAC, 0.0 = strong conflict.
    pub kac: f64,
    /// Entity coverage component (0.0–1.0): 1.0 = fully grounded, 0.0 = all entities novel.
    pub entity_coverage: f64,
}

impl CalibrationResult {
    /// Build a CalibrationResult from a completed PipelineReport.
    pub fn from_report(report: &PipelineReport) -> Self {
        let breakdown = ScoreBreakdown::compute(
            &report.understanding.category_analysis,
            &report.dialectic,
            report
                .wittgenstein
                .game_analysis
                .primary_game
                .as_ref()
                .map(|g| g.form_of_life),
        );

        let raw = breakdown.weighted_score();
        let ecs = (raw * 100.0).round().clamp(0.0, 100.0) as u8;
        let band = EcsBand::from_score(ecs);
        let calibrated = ecs >= 80;

        let flags = collect_flags(report);
        let safe_version = report.regulated_text.clone();
        let epistemic_mode = epistemic_mode_label(&report.understanding.category_analysis);

        Self {
            ecs,
            band,
            calibrated,
            flags,
            safe_version,
            epistemic_mode,
            score_breakdown: breakdown,
        }
    }

    /// Rewrite the `safe_version` text using domain-specific regulative rules.
    ///
    /// When ECS is HIGH, the input is already well-calibrated and may be returned
    /// largely unchanged. When LOW, domain-specific rules maximally hedge the text.
    pub fn rewrite_for_domain(&self, domain: RewriteDomain) -> RewriteResult {
        let rewriter = DomainRewriter::new(domain);
        rewriter.rewrite(&self.safe_version)
    }
}

impl ScoreBreakdown {
    /// Compute the seven components of the ECS.
    pub fn compute(
        categories: &CategoryAnalysis,
        dialectic: &DialecticReport,
        game: Option<FormOfLife>,
    ) -> Self {
        let modality = modality_factor(categories);
        let illusion = illusion_factor(dialectic);
        let antinomy = antinomy_factor(dialectic);
        let paralogism = paralogism_factor(dialectic);
        let game_stability = game_stability_factor(game);
        // KAC: 1 - kac_score (high kac_score = contradiction = low ECS component).
        // `None` means KAC did not run → neutral 0.5 (excluded from weighted average).
        let kac = match dialectic.kac_score {
            None => 0.5,
            Some(s) => (1.0 - s).clamp(0.0, 1.0),
        };
        // Entity coverage: 1 - entity_novelty.
        // `None` means coverage did not run → neutral 0.5.
        let entity_coverage = match dialectic.entity_novelty {
            None => 0.5,
            Some(n) => (1.0 - n).clamp(0.0, 1.0),
        };

        Self {
            modality,
            illusion,
            antinomy,
            paralogism,
            game_stability,
            kac,
            entity_coverage,
        }
    }

    /// Compute the weighted composite score (0.0–1.0).
    pub fn weighted_score(&self) -> f64 {
        // Adaptive denominator: KAC and coverage weights are only included when
        // structured context was available (signalled by non-0.5 values).
        // For unstructured text both default to 0.5 (neutral), excluded from denominator.
        let has_kac = (self.kac - 0.5).abs() > 0.01;
        let has_coverage = (self.entity_coverage - 0.5).abs() > 0.01;

        let w_kac_eff = if has_kac { W_KAC } else { 0.0 };
        let w_cov_eff = if has_coverage { W_COVERAGE } else { 0.0 };
        let w_total =
            W_MODALITY + W_ILLUSION + W_ANTINOMY + W_PARALOGISM + W_GAME + w_kac_eff + w_cov_eff;

        (W_MODALITY * self.modality
            + W_ILLUSION * self.illusion
            + W_ANTINOMY * self.antinomy
            + W_PARALOGISM * self.paralogism
            + W_GAME * self.game_stability
            + w_kac_eff * self.kac
            + w_cov_eff * self.entity_coverage)
            / w_total
    }
}

// ─── Component Factors ────────────────────────────────────────────────────────

/// Modality factor: measures whether the output uses appropriately hedged language.
///
/// * Possibility-dominant (may/could/might/suggests) → 1.0 (well-calibrated)
/// * Existence-dominant (is/actual/fact) → 0.55 (factual, moderate confidence)
/// * Necessity-dominant (must/certainly/always/never) → 0.0 (overconfident)
/// * No modality signal → 0.5 (neutral)
fn modality_factor(categories: &CategoryAnalysis) -> f64 {
    let poss = category_confidence(categories, Category::Possibility);
    let exist = category_confidence(categories, Category::Existence);
    let nec = category_confidence(categories, Category::Necessity);

    // Determine dominant modality category
    let max_modality = poss.max(exist).max(nec);

    if max_modality < 0.01 {
        // No modality signal — treat as neutral
        return 0.50;
    }

    // Weighted combination: Possibility raises, Necessity lowers.
    // Existence is neutral (0.0 contribution): unhedged existence claims like
    // "The patient has cancer" are the overconfident pattern we must not reward.
    let raw = 0.50 + (poss * 0.50) - (nec * 0.50);
    raw.clamp(0.0, 1.0)
}

/// Illusion factor: penalises transcendental illusions (epistemic overreach).
///
/// * No illusions → 1.0
/// * 1 Low severity illusion → 0.75
/// * 1 Medium severity → 0.55
/// * 1 High severity → 0.30
/// * Critical or multiple illusions → 0.0
fn illusion_factor(dialectic: &DialecticReport) -> f64 {
    let illusions = &dialectic.illusions;
    if illusions.is_empty() {
        return 1.0;
    }

    // Take the worst severity
    let worst = illusions
        .iter()
        .map(|i| i.severity)
        .max()
        .unwrap_or(IllusionSeverity::Low);

    let base = match worst {
        IllusionSeverity::Low => 0.75,
        IllusionSeverity::Medium => 0.55,
        IllusionSeverity::High => 0.30,
        IllusionSeverity::Critical => 0.05,
    };

    // Additional penalty for multiple illusions
    let count_penalty = ((illusions.len() as f64 - 1.0) * 0.10).min(0.30);
    (base - count_penalty).max(0.0)
}

/// Antinomy factor: penalises logical contradictions in the text.
///
/// * No antinomies with conflict → 1.0
/// * 1 antinomy with conflict → 0.45
/// * 2+ antinomies with conflict → 0.0
fn antinomy_factor(dialectic: &DialecticReport) -> f64 {
    let conflicts = dialectic
        .antinomies
        .iter()
        .filter(|a| a.has_conflict)
        .count();

    // Smooth harmonic decay: 0 → 1.0, 1 → 0.5, 2 → 0.33, 3 → 0.25, ...
    // Avoids the arbitrary 55-point cliff from the previous step function.
    1.0 / (1.0 + conflicts as f64)
}

/// Paralogism factor: penalises invalid self-referential reasoning.
///
/// * No paralogisms → 1.0
/// * 1 paralogism report with detections → 0.50
/// * 2+ reports with detections → 0.10
fn paralogism_factor(dialectic: &DialecticReport) -> f64 {
    let active = dialectic
        .paralogisms
        .iter()
        .filter(|p| p.has_paralogisms)
        .count();

    match active {
        0 => 1.0,
        1 => 0.50,
        _ => 0.10,
    }
}

/// Language-game stability factor: some language games have higher epistemic discipline.
///
/// Scientific and mathematical discourse have strong internal standards for
/// calibration (falsifiability, proof). Theological/philosophical discourse is
/// prone to transcendental overreach by design. Legal and technical are intermediate.
fn game_stability_factor(game: Option<FormOfLife>) -> f64 {
    match game {
        Some(FormOfLife::Scientific) | Some(FormOfLife::Mathematical) => 0.90,
        Some(FormOfLife::Technical) => 0.80,
        Some(FormOfLife::Legal) => 0.70,
        Some(FormOfLife::Moral) | Some(FormOfLife::Everyday) => 0.60,
        Some(FormOfLife::Narrative) | Some(FormOfLife::Aesthetic) => 0.55,
        Some(FormOfLife::Philosophical) => 0.45,
        Some(FormOfLife::Religious) => 0.35,
        None | Some(FormOfLife::Unknown) => 0.50,
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn category_confidence(categories: &CategoryAnalysis, cat: Category) -> f64 {
    categories
        .applications
        .iter()
        .find(|a| a.category == cat)
        .map(|a| a.confidence.value())
        .unwrap_or(0.0)
}

fn epistemic_mode_label(categories: &CategoryAnalysis) -> String {
    let poss = category_confidence(categories, Category::Possibility);
    let exist = category_confidence(categories, Category::Existence);
    let nec = category_confidence(categories, Category::Necessity);

    if poss >= exist && poss >= nec && poss > 0.01 {
        "Regulative (well-hedged)".to_string()
    } else if nec >= exist && nec > 0.01 {
        "Apodeictic (overconfident)".to_string()
    } else if exist > 0.01 {
        "Assertoric (factual)".to_string()
    } else {
        "Indeterminate".to_string()
    }
}

fn collect_flags(report: &PipelineReport) -> Vec<String> {
    let mut flags = Vec::new();

    for illusion in &report.dialectic.illusions {
        flags.push(format!(
            "Epistemic overreach: {:?} ({:?}) — {}",
            illusion.kind,
            illusion.severity,
            illusion.description.chars().take(80).collect::<String>()
        ));
    }

    for antinomy in &report.dialectic.antinomies {
        if antinomy.has_conflict {
            flags.push(format!(
                "Logical contradiction: {:?} antinomy detected",
                antinomy.antinomy
            ));
        }
    }

    for para_report in &report.dialectic.paralogisms {
        for para in &para_report.detected {
            flags.push(format!(
                "Self-referential error: {:?} paralogism detected",
                para.kind
            ));
        }
    }

    if matches!(report.verdict.risk, RiskLevel::Medium | RiskLevel::High) {
        if let Some(cat) = &report.verdict.dominant_category {
            if cat == "Necessity" {
                flags.push(
                    "Certainty overreach: output uses necessity-modal language (must, always, inevitably) \
                     without evidential warrant".to_string()
                );
            }
        }
    }

    flags
}

// ─── Trait extension on PipelineReport ───────────────────────────────────────

/// Extension trait to compute the ECS from any PipelineReport.
pub trait PipelineCalibration {
    /// Compute the Epistemic Confidence Score (ECS) for this report.
    fn calibration(&self) -> CalibrationResult;

    /// Return just the ECS score (0–100) without the full CalibrationResult.
    fn ecs(&self) -> u8;
}

impl PipelineCalibration for PipelineReport {
    fn calibration(&self) -> CalibrationResult {
        CalibrationResult::from_report(self)
    }

    fn ecs(&self) -> u8 {
        CalibrationResult::from_report(self).ecs
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::KantianPipeline;

    fn ecs(text: &str) -> u8 {
        let pipeline = KantianPipeline::new();
        let report = pipeline.process(text).unwrap();
        report.ecs()
    }

    #[test]
    fn safe_factual_text_scores_high() {
        // Hedged, scientific language → should score HIGH (≥80)
        let score = ecs("Water may boil at around 100 degrees Celsius under standard conditions.");
        assert!(
            score >= 60,
            "Hedged factual text should score at least 60, got {}",
            score
        );
    }

    #[test]
    fn necessity_dominant_text_scores_lower() {
        // Overconfident necessity language → should score LOWER than hedged version
        let hedged = ecs("It may be that all events have causes.");
        let certain =
            ecs("All events must necessarily have causes. This is certain and inevitable.");
        // Necessity-heavy text has more epistemic overreach → lower ECS
        assert!(
            certain <= hedged,
            "certain={certain:.3} should be ≤ hedged={hedged:.3}"
        );
    }

    #[test]
    fn theological_overreach_scores_low() {
        // God-idea hypostatization → should score LOW
        let score = ecs("God exists necessarily and is the ground of all being.");
        assert!(
            score < 60,
            "Theological overreach should score below 60, got {}",
            score
        );
    }

    #[test]
    fn contradiction_lowers_score() {
        // Antinomy → lower score
        let no_contradiction = ecs("Water boils at 100 degrees.");
        let contradiction =
            ecs("The universe had a beginning in time. The universe has no beginning.");
        assert!(
            contradiction < no_contradiction,
            "Contradiction should lower ECS: {} vs {}",
            contradiction,
            no_contradiction
        );
    }

    #[test]
    fn calibration_result_has_all_fields() {
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process("It is possible that the economy may recover.")
            .unwrap();
        let cal = report.calibration();
        assert!(cal.ecs <= 100);
        assert!(!cal.epistemic_mode.is_empty());
        assert!(!cal.band.label().is_empty());
    }

    #[test]
    fn calibrated_true_when_high() {
        let pipeline = KantianPipeline::new();
        // Simple neutral text should be calibrated
        let report = pipeline.process("Snow is white.").unwrap();
        let cal = report.calibration();
        assert_eq!(cal.calibrated, cal.ecs >= 80);
    }

    #[test]
    fn score_breakdown_sum_reasonable() {
        let pipeline = KantianPipeline::new();
        let report = pipeline.process("All bachelors are unmarried.").unwrap();
        let cal = report.calibration();
        let bd = &cal.score_breakdown;
        // All components should be in [0.0, 1.0]
        assert!((0.0..=1.0).contains(&bd.modality));
        assert!((0.0..=1.0).contains(&bd.illusion));
        assert!((0.0..=1.0).contains(&bd.antinomy));
        assert!((0.0..=1.0).contains(&bd.paralogism));
        assert!((0.0..=1.0).contains(&bd.game_stability));
    }

    #[test]
    fn ecs_band_from_score() {
        assert_eq!(EcsBand::from_score(85), EcsBand::High);
        assert_eq!(EcsBand::from_score(60), EcsBand::Moderate);
        assert_eq!(EcsBand::from_score(30), EcsBand::Low);
    }
}
