//! Causal Reasoning: Distinguish correlation from causation
//!
//! TRIZ Principle: Transition to Micro-Level + Feedback
//! Validate causal mechanisms, identify alternative causes,
//! and apply Bradford Hill criteria for causal evidence.

use serde::{Deserialize, Serialize};

/// Causal claim representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalClaim {
    /// Cause (independent variable)
    pub cause: String,
    /// Effect (dependent variable)
    pub effect: String,
    /// Proposed mechanism
    pub mechanism: String,
    /// Strength of evidence (0.0-1.0)
    pub evidence_strength: f64,
}

impl CausalClaim {
    /// Create a new causal claim
    pub fn new(cause: String, effect: String, mechanism: String, evidence_strength: f64) -> Self {
        Self {
            cause,
            effect,
            mechanism,
            evidence_strength: evidence_strength.clamp(0.0, 1.0),
        }
    }
}

/// Bradford Hill Criterion (used to evaluate causal evidence)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BradfordHillCriterion {
    /// Strength of association (relative risk ratio)
    Strength,
    /// Consistency across studies
    Consistency,
    /// Specificity of effect
    Specificity,
    /// Temporal relationship (cause before effect)
    Temporality,
    /// Biological gradient (dose-response)
    BiologicalGradient,
    /// Plausible mechanism
    Plausibility,
    /// Coherence with existing theory
    Coherence,
    /// Experiment evidence
    Experiment,
    /// Analogy to known causal mechanisms
    Analogy,
}

impl std::fmt::Display for BradfordHillCriterion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Strength => write!(f, "Strength"),
            Self::Consistency => write!(f, "Consistency"),
            Self::Specificity => write!(f, "Specificity"),
            Self::Temporality => write!(f, "Temporality"),
            Self::BiologicalGradient => write!(f, "Biological Gradient"),
            Self::Plausibility => write!(f, "Plausibility"),
            Self::Coherence => write!(f, "Coherence"),
            Self::Experiment => write!(f, "Experiment"),
            Self::Analogy => write!(f, "Analogy"),
        }
    }
}

/// Evidence for a Bradford Hill criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionEvidence {
    /// Which criterion
    pub criterion: BradfordHillCriterion,
    /// Met or not (true = meets criterion)
    pub met: bool,
    /// Strength of evidence (0.0-1.0)
    pub strength: f64,
    /// Explanation
    pub explanation: String,
}

impl CriterionEvidence {
    /// Create new evidence
    pub fn new(
        criterion: BradfordHillCriterion,
        met: bool,
        strength: f64,
        explanation: String,
    ) -> Self {
        Self {
            criterion,
            met,
            strength: strength.clamp(0.0, 1.0),
            explanation,
        }
    }
}

/// Alternative explanation for observed correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeCause {
    /// The alternative cause
    pub cause: String,
    /// How plausible this alternative is (0.0-1.0)
    pub plausibility: f64,
    /// Explanation
    pub reasoning: String,
}

/// Causal evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEvaluation {
    /// Original claim
    pub claim: CausalClaim,
    /// Bradford Hill criteria assessments
    pub criteria_evidence: Vec<CriterionEvidence>,
    /// Number of criteria met
    pub criteria_met: usize,
    /// Alternative causes identified
    pub alternative_causes: Vec<AlternativeCause>,
    /// Overall causal strength (0.0-1.0)
    pub causal_strength: f64,
    /// Confidence in causal relationship
    pub confidence: f64,
    /// Overall verdict (Likely/Possible/Unlikely)
    pub verdict: CausalVerdict,
}

/// Causal relationship verdict
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CausalVerdict {
    /// Strong evidence for causation
    LikelyCausal,
    /// Mixed evidence
    PossiblyRelated,
    /// Weak evidence for causation
    UnlikelyCausal,
    /// Evidence suggests confounding
    ProbablyConfounded,
}

impl std::fmt::Display for CausalVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LikelyCausal => write!(f, "Likely Causal"),
            Self::PossiblyRelated => write!(f, "Possibly Related"),
            Self::UnlikelyCausal => write!(f, "Unlikely Causal"),
            Self::ProbablyConfounded => write!(f, "Probably Confounded"),
        }
    }
}

/// Causal reasoning engine
pub struct CausalAnalyzer;

impl CausalAnalyzer {
    /// Evaluate a causal claim using Bradford Hill criteria
    pub fn evaluate(claim: &CausalClaim) -> CausalEvaluation {
        let mut criteria_evidence = vec![];

        // Simulate criterion evaluations based on evidence strength
        criteria_evidence.push(CriterionEvidence::new(
            BradfordHillCriterion::Strength,
            claim.evidence_strength > 0.6,
            claim.evidence_strength,
            format!(
                "Strength of association: {:.0}%",
                claim.evidence_strength * 100.0
            ),
        ));

        criteria_evidence.push(CriterionEvidence::new(
            BradfordHillCriterion::Temporality,
            claim.evidence_strength > 0.5,
            if claim.evidence_strength > 0.5 {
                0.85
            } else {
                0.4
            },
            "Temporal relationship between cause and effect".to_string(),
        ));

        criteria_evidence.push(CriterionEvidence::new(
            BradfordHillCriterion::Plausibility,
            !claim.mechanism.is_empty(),
            0.8,
            format!("Proposed mechanism: {}", claim.mechanism),
        ));

        criteria_evidence.push(CriterionEvidence::new(
            BradfordHillCriterion::Consistency,
            claim.evidence_strength > 0.65,
            if claim.evidence_strength > 0.65 {
                0.75
            } else {
                0.5
            },
            "Consistency across multiple studies".to_string(),
        ));

        criteria_evidence.push(CriterionEvidence::new(
            BradfordHillCriterion::BiologicalGradient,
            claim.evidence_strength > 0.70,
            if claim.evidence_strength > 0.70 {
                0.7
            } else {
                0.3
            },
            "Dose-response relationship".to_string(),
        ));

        criteria_evidence.push(CriterionEvidence::new(
            BradfordHillCriterion::Specificity,
            claim.evidence_strength > 0.75,
            if claim.evidence_strength > 0.75 {
                0.65
            } else {
                0.4
            },
            "Specificity of causal effect".to_string(),
        ));

        criteria_evidence.push(CriterionEvidence::new(
            BradfordHillCriterion::Coherence,
            claim.evidence_strength > 0.6,
            0.7,
            "Coherence with existing knowledge".to_string(),
        ));

        criteria_evidence.push(CriterionEvidence::new(
            BradfordHillCriterion::Experiment,
            claim.evidence_strength > 0.8,
            if claim.evidence_strength > 0.8 {
                0.85
            } else {
                0.3
            },
            "Experimental evidence available".to_string(),
        ));

        criteria_evidence.push(CriterionEvidence::new(
            BradfordHillCriterion::Analogy,
            claim.evidence_strength > 0.65,
            0.6,
            "Analogy to known mechanisms".to_string(),
        ));

        let criteria_met = criteria_evidence.iter().filter(|ce| ce.met).count();
        let avg_strength = criteria_evidence.iter().map(|ce| ce.strength).sum::<f64>()
            / criteria_evidence.len() as f64;

        let causal_strength =
            (avg_strength * 0.6 + (criteria_met as f64 / 9.0) * 0.4).clamp(0.0, 1.0);

        // Find alternative causes
        let alternative_causes = Self::identify_alternatives(claim);

        // Determine verdict
        let verdict = if criteria_met >= 7 && causal_strength > 0.75 {
            CausalVerdict::LikelyCausal
        } else if criteria_met >= 5 && causal_strength > 0.6 {
            CausalVerdict::PossiblyRelated
        } else if !alternative_causes.is_empty() && alternative_causes[0].plausibility > 0.7 {
            CausalVerdict::ProbablyConfounded
        } else {
            CausalVerdict::UnlikelyCausal
        };

        let confidence = causal_strength;

        CausalEvaluation {
            claim: claim.clone(),
            criteria_evidence,
            criteria_met,
            alternative_causes,
            causal_strength,
            confidence,
            verdict,
        }
    }

    /// Identify alternative explanations for correlation
    fn identify_alternatives(claim: &CausalClaim) -> Vec<AlternativeCause> {
        let mut alternatives = vec![];

        // Reverse causality
        alternatives.push(AlternativeCause {
            cause: format!("Reverse causality: {} → {}", claim.effect, claim.cause),
            plausibility: 0.4,
            reasoning: "Effect may cause the apparent cause instead".to_string(),
        });

        // Confounding variable
        alternatives.push(AlternativeCause {
            cause: "Confounding variable (unknown common cause)".to_string(),
            plausibility: 0.5,
            reasoning: "Third factor causes both apparent cause and effect".to_string(),
        });

        // Coincidence
        alternatives.push(AlternativeCause {
            cause: "Coincidence / Spurious correlation".to_string(),
            plausibility: 0.3,
            reasoning: "Correlation occurred by chance".to_string(),
        });

        alternatives
    }

    /// Generate explanation of causal evaluation
    pub fn explain(evaluation: &CausalEvaluation) -> String {
        let mut explanation = String::new();

        explanation.push_str("## Causal Analysis\n\n");
        explanation.push_str(&format!(
            "**Claim**: {} → {}\n**Mechanism**: {}\n**Evidence Strength**: {:.0}%\n\n",
            evaluation.claim.cause,
            evaluation.claim.effect,
            evaluation.claim.mechanism,
            evaluation.claim.evidence_strength * 100.0
        ));

        explanation.push_str("### Bradford Hill Criteria Assessment\n\n");
        for evidence in &evaluation.criteria_evidence {
            explanation.push_str(&format!(
                "- **{}**: {} ({:.0}%) - {}\n",
                evidence.criterion,
                if evidence.met {
                    "✓ Met"
                } else {
                    "✗ Not met"
                },
                evidence.strength * 100.0,
                evidence.explanation
            ));
        }

        explanation.push_str(&format!(
            "\n**Criteria Met**: {}/9\n",
            evaluation.criteria_met
        ));

        if !evaluation.alternative_causes.is_empty() {
            explanation.push_str("\n### Alternative Explanations\n\n");
            for alt in &evaluation.alternative_causes {
                explanation.push_str(&format!(
                    "- **{}** (plausibility: {:.0}%)\n  {}\n",
                    alt.cause,
                    alt.plausibility * 100.0,
                    alt.reasoning
                ));
            }
        }

        explanation.push_str(&format!(
            "\n### Verdict: {} (confidence: {:.0}%)\n",
            evaluation.verdict,
            evaluation.confidence * 100.0
        ));

        explanation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causal_claim_creation() {
        let claim = CausalClaim::new(
            "Smoking".to_string(),
            "Lung cancer".to_string(),
            "Carcinogens damage DNA".to_string(),
            0.85,
        );
        assert_eq!(claim.cause, "Smoking");
        assert_eq!(claim.evidence_strength, 0.85);
    }

    #[test]
    fn test_criterion_evidence_creation() {
        let ev = CriterionEvidence::new(
            BradfordHillCriterion::Temporality,
            true,
            0.90,
            "Smoking precedes cancer".to_string(),
        );
        assert!(ev.met);
        assert_eq!(ev.strength, 0.90);
    }

    #[test]
    fn test_causal_evaluation_strong() {
        let claim = CausalClaim::new(
            "Smoking".to_string(),
            "Cancer".to_string(),
            "Carcinogens".to_string(),
            0.85,
        );
        let eval = CausalAnalyzer::evaluate(&claim);
        assert!(eval.criteria_met > 5);
        assert!(eval.causal_strength > 0.6);
    }

    #[test]
    fn test_causal_evaluation_weak() {
        let claim = CausalClaim::new(
            "Color of car".to_string(),
            "Wealth".to_string(),
            "None".to_string(),
            0.2,
        );
        let eval = CausalAnalyzer::evaluate(&claim);
        assert!(eval.causal_strength < 0.5);
    }

    #[test]
    fn test_verdict_likely_causal() {
        let claim = CausalClaim::new(
            "High temp".to_string(),
            "Ice melts".to_string(),
            "Heat energy".to_string(),
            0.95,
        );
        let eval = CausalAnalyzer::evaluate(&claim);
        assert_eq!(eval.verdict, CausalVerdict::LikelyCausal);
    }

    #[test]
    fn test_alternative_causes() {
        let claim = CausalClaim::new(
            "Coffee".to_string(),
            "Alertness".to_string(),
            "Caffeine".to_string(),
            0.75,
        );
        let eval = CausalAnalyzer::evaluate(&claim);
        assert!(!eval.alternative_causes.is_empty());
    }

    #[test]
    fn test_bradford_hill_criterion_display() {
        assert_eq!(
            format!("{}", BradfordHillCriterion::Temporality),
            "Temporality"
        );
        assert_eq!(
            format!("{}", BradfordHillCriterion::Plausibility),
            "Plausibility"
        );
    }

    #[test]
    fn test_causal_verdict_display() {
        assert_eq!(format!("{}", CausalVerdict::LikelyCausal), "Likely Causal");
        assert_eq!(
            format!("{}", CausalVerdict::PossiblyRelated),
            "Possibly Related"
        );
    }

    #[test]
    fn test_explain_causal_evaluation() {
        let claim = CausalClaim::new(
            "A".to_string(),
            "B".to_string(),
            "Mechanism".to_string(),
            0.80,
        );
        let eval = CausalAnalyzer::evaluate(&claim);
        let explanation = CausalAnalyzer::explain(&eval);
        assert!(explanation.contains("Causal Analysis"));
        assert!(explanation.contains("Bradford Hill"));
        assert!(explanation.contains("Verdict"));
    }

    #[test]
    fn test_criteria_met_count() {
        let claim = CausalClaim::new("A".to_string(), "B".to_string(), "M".to_string(), 0.90);
        let eval = CausalAnalyzer::evaluate(&claim);
        assert!(eval.criteria_met <= 9);
        assert!(eval.criteria_met > 0);
    }

    #[test]
    fn test_confidence_range() {
        let claim = CausalClaim::new("A".to_string(), "B".to_string(), "M".to_string(), 0.5);
        let eval = CausalAnalyzer::evaluate(&claim);
        assert!(eval.confidence >= 0.0 && eval.confidence <= 1.0);
    }

    #[test]
    fn test_causal_strength_range() {
        let claim = CausalClaim::new("A".to_string(), "B".to_string(), "M".to_string(), 0.75);
        let eval = CausalAnalyzer::evaluate(&claim);
        assert!(eval.causal_strength >= 0.0 && eval.causal_strength <= 1.0);
    }
}
