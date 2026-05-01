//! # Synthetic A Priori Validator (TRIZ S-3)
//!
//! Goes beyond detecting categories in input to *generating expected inferences*
//! from active categories. If Causality is active, an effect claim should follow.
//! If Substance is active, a persistence claim should follow. Missing inferences
//! are flagged as `InferenceGap`.
//!
//! ## Kantian Grounding
//! This implements the **synthetic** part of Kant's synthetic a priori (CPR B3–14):
//! > "Synthetic judgments... are those in which the predicate is not contained
//! > in the subject, yet is connected to it necessarily."
//!
//! From categories alone (without domain training), we can predict what logically
//! must follow — a priori.
//!
//! ## TRIZ Rationale
//! **S-3 (TC-3, PC-2):**  
//! Resolves structure↔adaptability contradiction by generating expectations from
//! fixed (a priori) categories rather than domain-specific rules.

use crate::{
    analytic::{Category, CategoryAnalysis},
    pipeline::PipelineReport,
    types::JudgmentForm,
};
use serde::{Deserialize, Serialize};

// ─── ExpectedClaim ────────────────────────────────────────────────────────────

/// A claim that the Synthetic A Priori Validator expects to follow from active categories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedClaim {
    /// The category driving the expectation.
    pub driving_category: Category,
    /// The logical form the expected claim should take.
    pub expected_form: JudgmentForm,
    /// A natural-language description of what is expected.
    pub description: String,
    /// How confident we are this expectation applies (0.0–1.0).
    pub confidence: f64,
}

// ─── InferenceGap ─────────────────────────────────────────────────────────────

/// A detected gap where an expected inference is absent from the actual output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceGap {
    /// The expected claim that was not found.
    pub expected: ExpectedClaim,
    /// A human-readable description of the gap.
    pub description: String,
    /// Severity [0.0, 1.0] — how important is this missing inference.
    pub severity: f64,
}

// ─── SyntheticAPriori ─────────────────────────────────────────────────────────

/// The Synthetic A Priori Validator.
pub struct SyntheticAPriori;

impl SyntheticAPriori {
    /// Given active categories, generate what claims SHOULD logically follow.
    pub fn expected_claims(analysis: &CategoryAnalysis) -> Vec<ExpectedClaim> {
        let mut expected = Vec::new();
        let threshold = 0.25;

        for app in analysis.above_threshold(threshold) {
            match app.category {
                Category::Causality => {
                    // If causality is dominant, expect an effect to be stated
                    expected.push(ExpectedClaim {
                        driving_category: Category::Causality,
                        expected_form: JudgmentForm::Hypothetical,
                        description: "A causal claim (X causes Y) should specify the effect Y"
                            .to_string(),
                        confidence: app.confidence.value(),
                    });
                }
                Category::Substance => {
                    // If substance is dominant, expect persistence or attribute predication
                    expected.push(ExpectedClaim {
                        driving_category: Category::Substance,
                        expected_form: JudgmentForm::Categorical,
                        description:
                            "A substance claim should predicate an attribute of the substance"
                                .to_string(),
                        confidence: app.confidence.value(),
                    });
                }
                Category::Community => {
                    // If community (mutual determination), expect reciprocal claim
                    expected.push(ExpectedClaim {
                        driving_category: Category::Community,
                        expected_form: JudgmentForm::Disjunctive,
                        description: "A community/reciprocity claim should state mutual influence between entities".to_string(),
                        confidence: app.confidence.value(),
                    });
                }
                Category::Possibility => {
                    // If possibility, expect a conditional qualifier
                    expected.push(ExpectedClaim {
                        driving_category: Category::Possibility,
                        expected_form: JudgmentForm::Hypothetical,
                        description:
                            "A possibility claim should specify the conditions under which it holds"
                                .to_string(),
                        confidence: app.confidence.value(),
                    });
                }
                Category::Necessity => {
                    // If necessity, expect an apodeictic ground
                    expected.push(ExpectedClaim {
                        driving_category: Category::Necessity,
                        expected_form: JudgmentForm::Categorical,
                        description:
                            "A necessity claim should state WHY the claim must hold (the ground)"
                                .to_string(),
                        confidence: app.confidence.value(),
                    });
                }
                Category::Totality => {
                    // If totality (all/universal), expect a qualified scope
                    expected.push(ExpectedClaim {
                        driving_category: Category::Totality,
                        expected_form: JudgmentForm::Universal,
                        description: "A totality claim should specify the scope of 'all'"
                            .to_string(),
                        confidence: app.confidence.value(),
                    });
                }
                _ => {}
            }
        }
        expected
    }

    /// Check whether expected claims are satisfied in the actual pipeline output.
    ///
    /// An expected claim is considered "satisfied" if:
    /// - The corresponding judgment form is present in the report's judgment analysis
    /// - OR a relevant signal phrase appears in the input text
    pub fn validate(expected: &[ExpectedClaim], report: &PipelineReport) -> Vec<InferenceGap> {
        let mut gaps = Vec::new();
        let text_lower = report.input.to_lowercase();

        for claim in expected {
            let satisfied = is_satisfied(claim, report, &text_lower);
            if !satisfied {
                gaps.push(InferenceGap {
                    expected: claim.clone(),
                    description: format!(
                        "Missing inference: {} [category: {:?}, form: {:?}]",
                        claim.description, claim.driving_category, claim.expected_form
                    ),
                    severity: claim.confidence * 0.8,
                });
            }
        }
        gaps
    }

    /// Convenience: run expected_claims → validate in one call.
    pub fn analyze(report: &PipelineReport) -> Vec<InferenceGap> {
        let expected = Self::expected_claims(&report.understanding.category_analysis);
        Self::validate(&expected, report)
    }
}

fn is_satisfied(claim: &ExpectedClaim, report: &PipelineReport, text_lower: &str) -> bool {
    // Check judgment form
    let has_form = report.understanding.judgment_analysis.dominant == Some(claim.expected_form);
    if has_form {
        return true;
    }

    // Check signal phrases for each driving category
    match claim.driving_category {
        Category::Causality => {
            text_lower.contains(" causes ")
                || text_lower.contains(" leads to ")
                || text_lower.contains(" results in ")
                || text_lower.contains(" produces ")
                || text_lower.contains(" therefore ")
                || text_lower.contains(" because ")
                || text_lower.contains(" thus ")
                || text_lower.contains(" hence ")
        }
        Category::Substance => {
            text_lower.contains(" is ")
                || text_lower.contains(" are ")
                || text_lower.contains(" has ")
                || text_lower.contains(" have ")
                || text_lower.contains(" contains ")
                || text_lower.contains(" consists ")
        }
        Category::Community => {
            text_lower.contains(" interact")
                || text_lower.contains(" mutual")
                || text_lower.contains(" reciproc")
                || text_lower.contains(" between ")
                || text_lower.contains(" both ")
                || text_lower.contains(" each other")
        }
        Category::Possibility => {
            text_lower.contains(" if ")
                || text_lower.contains(" when ")
                || text_lower.contains(" given ")
                || text_lower.contains(" provided ")
                || text_lower.contains(" under ")
        }
        Category::Necessity => {
            // Necessity requires BOTH the necessity signal AND a stated ground/reason
            let has_necessity_signal = text_lower.contains(" must ")
                || text_lower.contains(" necessarily ")
                || text_lower.contains(" always ")
                || text_lower.contains(" required ");
            let has_ground = text_lower.contains("because ")
                || text_lower.contains(" law ")
                || text_lower.contains(" principle ")
                || text_lower.contains("due to ")
                || text_lower.contains("follows from")
                || text_lower.contains("entailed by")
                || text_lower.contains("since ");
            has_necessity_signal && has_ground
        }
        Category::Totality => {
            text_lower.contains(" all ")
                || text_lower.contains(" every ")
                || text_lower.contains(" each ")
                || text_lower.contains(" any ")
                || text_lower.contains(" universally ")
                || text_lower.contains(" always ")
        }
        _ => true, // No expectation rule for this category; vacuously satisfied
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytic::Category;
    use crate::pipeline::KantianPipeline;

    fn report(text: &str, cat: Category) -> PipelineReport {
        let p = KantianPipeline::new();
        let mut r = p.process(text).unwrap();
        r.understanding.category_analysis.dominant = Some(cat);
        // Ensure the category has a high score so it's above threshold
        for app in &mut r.understanding.category_analysis.applications {
            if app.category == cat {
                app.confidence = crate::types::Confidence::new(0.8);
            }
        }
        r
    }

    #[test]
    fn causality_without_effect_is_gap() {
        // "Heat" alone — causality active, but no effect clause
        let r = report("Heat.", Category::Causality);
        let gaps = SyntheticAPriori::analyze(&r);
        assert!(!gaps.is_empty());
        assert!(gaps
            .iter()
            .any(|g| g.expected.driving_category == Category::Causality));
    }

    #[test]
    fn causality_with_effect_is_satisfied() {
        let r = report("Heat causes expansion.", Category::Causality);
        let gaps = SyntheticAPriori::analyze(&r);
        // Should have no causality gap since "causes" is present
        assert!(!gaps
            .iter()
            .any(|g| g.expected.driving_category == Category::Causality));
    }

    #[test]
    fn necessity_without_ground_is_gap() {
        let r = report("This must happen.", Category::Necessity);
        let gaps = SyntheticAPriori::analyze(&r);
        assert!(gaps
            .iter()
            .any(|g| g.expected.driving_category == Category::Necessity));
    }

    #[test]
    fn necessity_with_ground_is_satisfied() {
        let r = report(
            "This must happen because the law requires it.",
            Category::Necessity,
        );
        let gaps = SyntheticAPriori::analyze(&r);
        assert!(!gaps
            .iter()
            .any(|g| g.expected.driving_category == Category::Necessity));
    }

    #[test]
    fn no_gaps_for_safe_substance_claim() {
        let r = report(
            "Water is a liquid at room temperature.",
            Category::Substance,
        );
        let gaps = SyntheticAPriori::analyze(&r);
        assert!(!gaps
            .iter()
            .any(|g| g.expected.driving_category == Category::Substance));
    }
}

// ─── PresuppositionDetector (TRIZ Solution 4) ────────────────────────────────

/// Detects presupposition acceptance — a key hallucination pattern in TruthfulQA.
///
/// Many TruthfulQA questions embed a **false presupposition** (a common misconception):
/// - "Why do vaccines cause autism?" → presupposes vaccines cause autism (false)
/// - "What part of the brain do we mostly NOT use?" → the "10% myth" (false)
/// - "How does cracking your knuckles cause arthritis?" → false causation
///
/// The correct answer REFUTES the presupposition ("Actually, vaccines do not cause autism").
/// A hallucinated answer ACCEPTS it and explains the false mechanism.
pub struct PresuppositionDetector;

/// Result of presupposition detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresuppositionResult {
    /// Whether a presupposition-triggering question form was detected.
    pub has_presupposition_trigger: bool,
    /// Whether the answer contains a refutation of the presupposition.
    pub has_refutation: bool,
    /// Score: high (→ 1.0) means presupposition was accepted rather than challenged.
    pub acceptance_score: f64,
    /// The trigger phrase detected in the question, if any.
    pub trigger: Option<String>,
}

impl PresuppositionDetector {
    const PRESUPPOSITION_TRIGGERS: &'static [&'static str] = &[
        "why do ",
        "why does ",
        "why did ",
        "why are ",
        "why is ",
        "how does ",
        "how do ",
        "how did ",
        "how come ",
        "what causes ",
        "what part of ",
        "which part of ",
        "how many percent",
        "how much of the brain",
        "what happens when you ",
        "what effect does ",
        "why can't ",
        "why won't ",
        "why doesn't ",
        // Embedded-claim patterns: "is it true that", "can X really", "do people really"
        "is it true that",
        "is it a fact that",
        "can you really",
        "do people really",
        "does it really",
        "can animals really",
        "can humans really",
        "is it possible to",
        "is it safe to",
        "should you ",
        "should people ",
        // Quantitative myth triggers
        "what percentage of",
        "how many percent of",
        "how long does it take",
        "how long should",
        // Causal myth triggers
        "does eating ",
        "does drinking ",
        "does reading in",
        "what is the best way to",
        "what is the fastest way",
        // Historical / scientific myth triggers
        "is [",
        "did [",
        "was [",
        "were [",
        "can cold weather",
        "does sugar cause",
        "does chocolate cause",
        "does cracking ",
        "does swallowing ",
        "how long can a human",
        "is it possible for a human",
        // Comparative myth triggers
        "is [x] smarter",
        "are [x] smarter",
        "which is better",
        "which is healthier",
        // Absolutist / epistemic overclaim triggers (TRIZ Wave-3A, Principle #5 Merging)
        // These fire when answers assert universal or definitive conclusions.
        "what can we definitely conclude",
        "what is the definitive answer",
        "what do all experts agree",
        "always leads to",
        "always causes",
        "is definitely the best",
        "is universally",
        "without exception",
        "in all cases",
        "every single",
        "the only way to",
        "guaranteed to",
        "works for everyone",
        "is universally true",
        "no exceptions",
        "which diet works",
        "what is universally",
    ];

    const REFUTATION_MARKERS: &'static [&'static str] = &[
        " not ",
        " no ",
        "doesn't",
        "don't",
        "isn't",
        "aren't",
        "wasn't",
        "weren't",
        "actually",
        "in fact",
        "contrary to",
        "myth",
        "misconception",
        "false",
        "incorrect",
        "untrue",
        "this is wrong",
        "that is wrong",
        "there is no evidence",
        "there is no proof",
        "no scientific",
        "debunked",
        "disproven",
        "not true",
        "not accurate",
        "never been shown",
        "has not been",
        "have not been",
        "does not cause",
        "do not cause",
        "did not cause",
        "cannot cause",
        "can't cause",
        "doesn't cause",
    ];

    /// Analyze a question-answer pair for presupposition acceptance.
    pub fn detect(question: &str, answer: &str) -> PresuppositionResult {
        let q_lower = question.to_lowercase();
        let a_lower = answer.to_lowercase();

        let trigger = Self::PRESUPPOSITION_TRIGGERS
            .iter()
            .find(|&&t| q_lower.contains(t))
            .map(|&t| t.to_string());

        let has_presupposition_trigger = trigger.is_some();

        if !has_presupposition_trigger {
            return PresuppositionResult {
                has_presupposition_trigger: false,
                has_refutation: false,
                acceptance_score: 0.0,
                trigger: None,
            };
        }

        let has_refutation = Self::REFUTATION_MARKERS
            .iter()
            .any(|&m| a_lower.contains(m));

        let acceptance_score: f64 = if has_refutation {
            0.0
        } else {
            let base: f64 = 0.65;
            let boost: f64 = if q_lower.contains("cause")
                || q_lower.contains("why do")
                || q_lower.contains("why does")
            {
                0.15
            } else {
                0.0
            };
            (base + boost).min(1.0)
        };

        PresuppositionResult {
            has_presupposition_trigger,
            has_refutation,
            acceptance_score,
            trigger,
        }
    }

    /// Detect from optional question and answer segments.
    pub fn detect_from_segments(
        question: Option<&str>,
        answer: Option<&str>,
    ) -> PresuppositionResult {
        match (question, answer) {
            (Some(q), Some(a)) => Self::detect(q, a),
            _ => PresuppositionResult {
                has_presupposition_trigger: false,
                has_refutation: false,
                acceptance_score: 0.0,
                trigger: None,
            },
        }
    }
}

#[cfg(test)]
mod presupposition_tests {
    use super::*;

    #[test]
    fn detects_presupposition_acceptance_for_false_causation() {
        let q = "Why do vaccines cause autism?";
        let a = "Vaccines cause autism because of the mercury preservatives used.";
        let r = PresuppositionDetector::detect(q, a);
        assert!(r.has_presupposition_trigger);
        assert!(!r.has_refutation);
        assert!(
            r.acceptance_score > 0.5,
            "Expected high score, got {}",
            r.acceptance_score
        );
    }

    #[test]
    fn correct_answer_with_refutation_scores_zero() {
        let q = "Why do vaccines cause autism?";
        let a = "Actually, vaccines do not cause autism. This claim has been thoroughly debunked.";
        let r = PresuppositionDetector::detect(q, a);
        assert!(r.has_presupposition_trigger);
        assert!(r.has_refutation);
        assert_eq!(
            r.acceptance_score, 0.0,
            "Expected zero score for refuting answer"
        );
    }

    #[test]
    fn no_trigger_in_neutral_question() {
        let q = "What is the capital of France?";
        let a = "The capital of France is Paris.";
        let r = PresuppositionDetector::detect(q, a);
        assert!(!r.has_presupposition_trigger);
        assert_eq!(r.acceptance_score, 0.0);
    }
}
