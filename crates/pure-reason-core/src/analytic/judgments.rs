//! # Table of Judgments
//!
//! Kant's Table of Judgments is the logical foundation from which the Categories
//! are derived. Each of the 12 logical forms of judgment corresponds to one of
//! the 12 Categories.
//!
//! "The same understanding, and indeed by means of the very same actions through
//! which it brings the logical form of a judgment into concepts by means of the
//! analytical unity, also brings a transcendental content into its representations."
//! — Kant, CPR B105

use crate::types::{Confidence, JudgmentForm, JudgmentGroup, Proposition};
use serde::{Deserialize, Serialize};

// ─── JudgmentDetection ───────────────────────────────────────────────────────

/// A detected judgment form in a proposition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgmentDetection {
    pub form: JudgmentForm,
    pub confidence: Confidence,
    /// Evidence phrases found in the text.
    pub evidence: Vec<String>,
}

// ─── JudgmentDetector ────────────────────────────────────────────────────────

/// Detects judgment forms in propositions using linguistic heuristics.
pub struct JudgmentDetector;

impl JudgmentDetector {
    /// Signal words / patterns for each judgment form.
    fn signals(form: JudgmentForm) -> &'static [&'static str] {
        match form {
            // Quantity
            JudgmentForm::Universal => &[
                "all",
                "every",
                "each",
                "always",
                "universally",
                "without exception",
            ],
            JudgmentForm::Particular => &[
                "some",
                "several",
                "certain",
                "many",
                "a few",
                "sometimes",
                "particular",
            ],
            JudgmentForm::Singular => &["this", "that", "the specific", "one particular", "here"],

            // Quality
            JudgmentForm::Affirmative => &["is", "are", "was", "were", "has", "have", "does"],
            JudgmentForm::Negative => &[
                "is not", "are not", "never", "no", "nothing", "none", "neither", "nor",
            ],
            JudgmentForm::Infinite => &["is non-", "not-", "other than", "is something other"],

            // Relation
            JudgmentForm::Categorical => &["is", "are", "constitutes", "defines", "characterizes"],
            JudgmentForm::Hypothetical => &[
                "if",
                "then",
                "when",
                "provided that",
                "given that",
                "assuming",
                "suppose",
            ],
            JudgmentForm::Disjunctive => &["either", "or", "alternatively", "one of", "whether"],

            // Modality
            JudgmentForm::Problematic => &[
                "possibly",
                "might",
                "could",
                "perhaps",
                "maybe",
                "it is possible",
            ],
            JudgmentForm::Assertoric => &[
                "is",
                "actually",
                "in fact",
                "indeed",
                "certainly",
                "definitely",
            ],
            JudgmentForm::Apodeictic => &[
                "must",
                "necessarily",
                "cannot but",
                "it is necessary",
                "inevitably",
                "always",
            ],
        }
    }

    /// Detect a single judgment form in a proposition.
    pub fn detect_form(form: JudgmentForm, proposition: &Proposition) -> JudgmentDetection {
        let text = proposition.text.to_lowercase();
        let signals = Self::signals(form);

        let evidence: Vec<String> = signals
            .iter()
            .filter(|&&signal| text.contains(signal))
            .map(|&s| s.to_string())
            .collect();

        let confidence = if evidence.is_empty() {
            Confidence::new(0.0)
        } else {
            Confidence::new((evidence.len() as f64 / signals.len() as f64).min(1.0))
        };

        JudgmentDetection {
            form,
            confidence,
            evidence,
        }
    }

    /// Detect all 12 judgment forms in a proposition.
    pub fn detect_all(proposition: &Proposition) -> Vec<JudgmentDetection> {
        JudgmentForm::all()
            .iter()
            .map(|&form| Self::detect_form(form, proposition))
            .collect()
    }
}

// ─── JudgmentAnalysis ────────────────────────────────────────────────────────

/// The result of analyzing judgment forms across a set of propositions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgmentAnalysis {
    pub detections: Vec<JudgmentDetection>,
    /// The most prominent judgment form.
    pub dominant: Option<JudgmentForm>,
    /// Per-group summary.
    pub group_scores: Vec<GroupScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupScore {
    pub group: JudgmentGroup,
    pub score: f64,
}

impl JudgmentAnalysis {
    /// Analyze a set of propositions for their judgment forms.
    pub fn from_propositions(propositions: &[Proposition]) -> Self {
        if propositions.is_empty() {
            return Self {
                detections: Vec::new(),
                dominant: None,
                group_scores: Vec::new(),
            };
        }

        // Aggregate scores across all propositions for each judgment form
        let mut form_scores: Vec<(JudgmentForm, f64, Vec<String>)> = JudgmentForm::all()
            .iter()
            .map(|&form| {
                let mut total_score = 0.0;
                let mut all_evidence: Vec<String> = Vec::new();

                for prop in propositions {
                    let det = JudgmentDetector::detect_form(form, prop);
                    total_score += det.confidence.value();
                    all_evidence.extend(det.evidence);
                }

                let avg = total_score / propositions.len() as f64;
                all_evidence.sort();
                all_evidence.dedup();
                (form, avg, all_evidence)
            })
            .collect();

        form_scores.sort_by(|a, b| b.1.total_cmp(&a.1));

        let dominant = form_scores
            .first()
            .filter(|(_, score, _)| *score > 0.0)
            .map(|(form, _, _)| *form);

        let detections: Vec<JudgmentDetection> = form_scores
            .iter()
            .map(|(form, score, evidence)| JudgmentDetection {
                form: *form,
                confidence: Confidence::new(*score),
                evidence: evidence.clone(),
            })
            .collect();

        // Per-group scores
        let group_scores = [
            JudgmentGroup::Quantity,
            JudgmentGroup::Quality,
            JudgmentGroup::Relation,
            JudgmentGroup::Modality,
        ]
        .iter()
        .map(|&group| {
            let group_forms: Vec<JudgmentForm> = JudgmentForm::all()
                .iter()
                .filter(|f| f.group() == group)
                .copied()
                .collect();
            let score = detections
                .iter()
                .filter(|d| group_forms.contains(&d.form))
                .map(|d| d.confidence.value())
                .sum::<f64>()
                / group_forms.len() as f64;
            GroupScore { group, score }
        })
        .collect();

        Self {
            detections,
            dominant,
            group_scores,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PropositionKind;

    fn prop(text: &str) -> Proposition {
        Proposition::new(text, PropositionKind::Unknown)
    }

    #[test]
    fn hypothetical_detected() {
        let p = prop("If it rains, then the ground will be wet");
        let all = JudgmentDetector::detect_all(&p);
        let hyp = all
            .iter()
            .find(|d| d.form == JudgmentForm::Hypothetical)
            .unwrap();
        assert!(hyp.confidence.value() > 0.0);
    }

    #[test]
    fn universal_detected() {
        let p = prop("All humans are mortal and every person dies");
        let all = JudgmentDetector::detect_all(&p);
        let univ = all
            .iter()
            .find(|d| d.form == JudgmentForm::Universal)
            .unwrap();
        assert!(univ.confidence.value() > 0.0);
    }

    #[test]
    fn analysis_has_dominant() {
        let props = vec![
            prop("If the temperature drops, then ice forms"),
            prop("If it snows, then roads become slippery"),
        ];
        let analysis = JudgmentAnalysis::from_propositions(&props);
        assert!(analysis.dominant.is_some());
    }
}
