//! # Ideal of Pure Reason
//!
//! The Ideal of Pure Reason is Kant's analysis of the concept of God as the
//! ens realissimum (most real being) — the unconditioned ground of all conditions.
//!
//! Kant analyses three traditional proofs of God's existence and shows they all fail:
//!
//! 1. **Ontological Argument** — from the concept of a perfect being to its existence.
//!    Kant's refutation: "Existence is not a predicate."
//!
//! 2. **Cosmological Argument** — from contingent existence to a necessary being.
//!    Kant's refutation: it smuggles in the ontological argument.
//!
//! 3. **Physico-Theological Argument** (Design Argument) — from design in nature to a designer.
//!    Kant's refutation: even if proven, it only proves a demiurge, not the ens realissimum.
//!
//! Nevertheless, the Ideal of Pure Reason has a legitimate **regulative** use:
//! it serves as the idea of a perfect, unified system of knowledge — a regulative goal.

use crate::types::Proposition;
use serde::{Deserialize, Serialize};

pub use crate::dialectic::ideas::TheologicalIdea;

// ─── TheoreticalProof ────────────────────────────────────────────────────────

/// One of the three traditional proofs of God's existence, as analyzed by Kant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TheoreticalProof {
    /// The ontological argument: from the concept of perfection to existence.
    Ontological,
    /// The cosmological argument: from contingent existence to necessary being.
    Cosmological,
    /// The physico-theological (design) argument: from design in nature to a designer.
    PhysicoTheological,
}

impl TheoreticalProof {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Ontological => "Ontological Argument",
            Self::Cosmological => "Cosmological Argument",
            Self::PhysicoTheological => "Physico-Theological (Design) Argument",
        }
    }

    pub fn formulation(&self) -> &'static str {
        match self {
            Self::Ontological => {
                "God is the most perfect being. Existence is a perfection. \
                 Therefore, God must exist (existence follows from the concept of perfection)."
            }
            Self::Cosmological => {
                "Contingent things exist. Every contingent thing has a cause. \
                 The chain of causes cannot be infinite. Therefore, a necessary being exists."
            }
            Self::PhysicoTheological => {
                "The world exhibits order and design. \
                 Order and design imply an intelligent designer. \
                 Therefore, an intelligent designer (God) exists."
            }
        }
    }

    pub fn kantian_refutation(&self) -> &'static str {
        match self {
            Self::Ontological =>
                "Existence is not a predicate. No concept can guarantee its own instantiation. \
                 'A hundred actual thalers do not contain the least bit more than a hundred possible thalers.' (CPR A599/B627)",
            Self::Cosmological =>
                "The argument uses the category of causality beyond its valid range (experience). \
                 It also covertly relies on the ontological argument to establish necessity from contingency.",
            Self::PhysicoTheological =>
                "Even if valid, it proves only a very powerful architect, not the ens realissimum. \
                 It then relies on the cosmological argument, which in turn relies on the ontological argument.",
        }
    }

    pub fn signal_phrases(&self) -> &'static [&'static str] {
        match self {
            Self::Ontological => &[
                "god must exist by definition",
                "existence is part of god's nature",
                "perfect being must exist",
                "god cannot not exist",
                "necessary existence",
                "existence follows from perfection",
                // Common formulations of the ontological argument
                "god necessarily exists",
                "god exists necessarily",
                "most perfect being",
                "ground of all being",
                "ens realissimum",
                "omnibenevolent and omnipotent",
            ],
            Self::Cosmological => &[
                "first cause",
                "uncaused cause",
                "something must have started it all",
                "infinite regress is impossible",
                "necessary being caused everything",
                "why is there something rather than nothing",
            ],
            Self::PhysicoTheological => &[
                "the universe shows design",
                "fine-tuning",
                "irreducible complexity",
                "too ordered to be random",
                "designed by an intelligence",
                "cosmic designer",
                "watchmaker",
            ],
        }
    }
}

// ─── IdealDetection ──────────────────────────────────────────────────────────

/// A detection of a theological argument in text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdealDetection {
    pub proof: TheoreticalProof,
    pub proposition: Proposition,
    pub trigger: String,
    pub refutation: String,
    pub regulative_interpretation: String,
}

// ─── IdealOfPureReason ───────────────────────────────────────────────────────

/// The Ideal of Pure Reason — God as the ens realissimum (most real being).
///
/// Provides analysis of theological arguments and their Kantian refutations,
/// while also offering the legitimate regulative interpretation.
pub struct IdealOfPureReason;

impl IdealOfPureReason {
    /// Detect theological arguments in propositions and provide Kantian analysis.
    pub fn analyze(propositions: &[Proposition]) -> Vec<IdealDetection> {
        let mut detections = Vec::new();

        for prop in propositions {
            let text = prop.text.to_lowercase();

            for proof in [
                TheoreticalProof::Ontological,
                TheoreticalProof::Cosmological,
                TheoreticalProof::PhysicoTheological,
            ] {
                for &phrase in proof.signal_phrases() {
                    if text.contains(phrase) {
                        detections.push(IdealDetection {
                            proof,
                            proposition: prop.clone(),
                            trigger: phrase.to_string(),
                            refutation: proof.kantian_refutation().to_string(),
                            regulative_interpretation: Self::regulative_interpretation(proof),
                        });
                        break;
                    }
                }
            }
        }

        detections
    }

    fn regulative_interpretation(proof: TheoreticalProof) -> String {
        match proof {
            TheoreticalProof::Ontological =>
                "Regulatively: the Ideal of Pure Reason functions as the idea of a completely determined \
                 system of knowledge — an ideal of theoretical completeness that guides inquiry without \
                 being an actual object.",
            TheoreticalProof::Cosmological =>
                "Regulatively: the idea of a necessary ground of all contingent things \
                 functions as a regulative principle guiding us to seek ever-deeper explanations, \
                 without positing an actual necessary being.",
            TheoreticalProof::PhysicoTheological =>
                "Regulatively: the idea of nature as if designed by an intelligence \
                 is a valuable heuristic for biological and natural inquiry (teleological judgment), \
                 without asserting an actual designer.",
        }.to_string()
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
    fn ontological_argument_detected() {
        let p = prop("God must exist by definition since existence follows from perfection");
        let detections = IdealOfPureReason::analyze(&[p]);
        assert!(detections
            .iter()
            .any(|d| d.proof == TheoreticalProof::Ontological));
    }

    #[test]
    fn cosmological_argument_detected() {
        let p = prop("There must be a first cause since infinite regress is impossible");
        let detections = IdealOfPureReason::analyze(&[p]);
        assert!(detections
            .iter()
            .any(|d| d.proof == TheoreticalProof::Cosmological));
    }

    #[test]
    fn clean_text_has_no_detections() {
        let p = prop("Hydrogen and oxygen combine to form water molecules");
        let detections = IdealOfPureReason::analyze(&[p]);
        assert!(detections.is_empty());
    }
}
