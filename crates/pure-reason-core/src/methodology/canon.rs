//! # Canon of Pure Reason
//!
//! The Canon of Pure Reason identifies the legitimate positive uses of Pure Reason.
//! While the Discipline says what Reason must NOT do, the Canon says what it CAN do.
//!
//! Kant's conclusion: Pure Reason has no legitimate theoretical (speculative) use
//! beyond the bounds of possible experience. Its legitimate use is PRACTICAL:
//! in the domain of freedom, morality, and the highest good.
//!
//! The three "questions" the Canon addresses:
//! 1. What can I know? (Theoretical use — limited to phenomena)
//! 2. What ought I to do? (Practical use — moral law)
//! 3. What may I hope? (Practical/Theological use — immortality, God as moral postulates)

use crate::types::Proposition;
use serde::{Deserialize, Serialize};

// ─── PracticalUse ────────────────────────────────────────────────────────────

/// A legitimate practical use of Pure Reason.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticalUse {
    pub kind: PracticalUseKind,
    pub description: String,
    pub proposition: Option<Proposition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PracticalUseKind {
    /// Moral reasoning — "What ought I to do?"
    Moral,
    /// Epistemic humility — acknowledging limits of knowledge.
    EpistemicHumility,
    /// Regulative use of Ideas — guiding inquiry without claiming objects.
    Regulative,
    /// Systematic unity — seeking coherence and completeness in knowledge.
    SystematicUnity,
    /// Practical postulate — positing God, freedom, or immortality for moral reasons.
    PracticalPostulate,
}

// ─── Canon ───────────────────────────────────────────────────────────────────

/// The Canon of Pure Reason — identifies legitimate uses of reason.
pub struct Canon;

impl Canon {
    pub fn new() -> Self {
        Self
    }

    /// The three canonical questions.
    pub fn canonical_questions() -> [(&'static str, &'static str); 3] {
        [
            ("What can I know?", "Theoretical use: limited to objects of possible experience (phenomena). Mathematics and pure natural science are genuine knowledge."),
            ("What ought I to do?", "Practical use: the moral law (categorical imperative) is the legitimate domain of Pure Reason in its practical employment."),
            ("What may I hope?", "Practical-theological use: God, freedom, and immortality are postulates of practical reason — not theoretical knowledge, but moral faith."),
        ]
    }

    /// Identify legitimate uses of reason in propositions.
    pub fn identify_legitimate_uses(&self, propositions: &[Proposition]) -> Vec<PracticalUse> {
        let mut uses = Vec::new();

        for prop in propositions {
            let text = prop.text.to_lowercase();

            // Moral reasoning
            if self.contains_any(
                &text,
                &[
                    "ought",
                    "should",
                    "morally",
                    "ethics",
                    "duty",
                    "right action",
                    "categorical",
                ],
            ) {
                uses.push(PracticalUse {
                    kind: PracticalUseKind::Moral,
                    description: "Moral/practical reasoning — a legitimate use of Pure Reason"
                        .to_string(),
                    proposition: Some(prop.clone()),
                });
            }

            // Epistemic humility
            if self.contains_any(
                &text,
                &[
                    "we cannot know",
                    "beyond our knowledge",
                    "limits of understanding",
                    "unknowable in principle",
                    "exceeds our experience",
                ],
            ) {
                uses.push(PracticalUse {
                    kind: PracticalUseKind::EpistemicHumility,
                    description: "Epistemic humility — acknowledging limits of knowledge, a core Kantian virtue".to_string(),
                    proposition: Some(prop.clone()),
                });
            }

            // Regulative use
            if self.contains_any(
                &text,
                &[
                    "as if",
                    "guide",
                    "ideal of",
                    "regulative",
                    "heuristic",
                    "maxim of inquiry",
                ],
            ) {
                uses.push(PracticalUse {
                    kind: PracticalUseKind::Regulative,
                    description: "Regulative use of Reason — treating Ideas as guiding ideals, not real objects".to_string(),
                    proposition: Some(prop.clone()),
                });
            }

            // Systematic unity
            if self.contains_any(
                &text,
                &[
                    "unified theory",
                    "systematic",
                    "coherent whole",
                    "unified account",
                    "comprehensive",
                ],
            ) {
                uses.push(PracticalUse {
                    kind: PracticalUseKind::SystematicUnity,
                    description: "Seeking systematic unity — a legitimate goal of reason"
                        .to_string(),
                    proposition: Some(prop.clone()),
                });
            }
        }

        uses
    }

    fn contains_any(&self, text: &str, terms: &[&str]) -> bool {
        terms.iter().any(|&t| text.contains(t))
    }
}

impl Default for Canon {
    fn default() -> Self {
        Self::new()
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
    fn three_canonical_questions() {
        assert_eq!(Canon::canonical_questions().len(), 3);
    }

    #[test]
    fn moral_reasoning_identified() {
        let p = prop("We ought to act according to duty and moral principles");
        let canon = Canon::new();
        let uses = canon.identify_legitimate_uses(&[p]);
        assert!(uses.iter().any(|u| u.kind == PracticalUseKind::Moral));
    }
}
