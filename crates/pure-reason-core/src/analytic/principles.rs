//! # Principles of Pure Understanding
//!
//! The Principles of Pure Understanding are the highest rules for the application
//! of categories to experience. They are synthetic a priori propositions that
//! constitute the framework of possible experience.
//!
//! ## The Four Principles:
//!
//! 1. **Axioms of Intuition** — "All intuitions are extensive magnitudes"
//!    All appearances have extensive magnitude (they occupy space and time).
//!
//! 2. **Anticipations of Perception** — "In all appearances sensation has intensive magnitude"
//!    All sensation has a degree (intensive magnitude) that can vary from 0 to maximum.
//!
//! 3. **Analogies of Experience** — "Experience is possible only through the representation of a necessary connection of perceptions"
//!    - First Analogy (Substance): "In all change of appearances, substance is permanent"
//!    - Second Analogy (Causality): "All alterations take place in conformity with the rule of cause and effect"
//!    - Third Analogy (Community): "All substances, insofar as they can be perceived as simultaneous in space, are in thoroughgoing reciprocity"
//!
//! 4. **Postulates of Empirical Thought** — Conditions of possibility, actuality, necessity:
//!    - Possibility: agreement with formal conditions of experience
//!    - Actuality: connection with material conditions of experience (sensation)
//!    - Necessity: determination by universal conditions of experience

use crate::types::Proposition;
use serde::{Deserialize, Serialize};

// ─── Principle ───────────────────────────────────────────────────────────────

/// One of the four Principles of Pure Understanding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Principle {
    /// Axioms of Intuition — all intuitions are extensive magnitudes.
    AxiomsOfIntuition,
    /// Anticipations of Perception — all sensation has intensive magnitude (degree).
    AnticipationsOfPerception,
    /// Analogies of Experience — Substance, Causality, Community.
    AnalogiesOfExperience,
    /// Postulates of Empirical Thought — Possibility, Actuality, Necessity.
    PostulatesOfEmpiricalThought,
}

impl Principle {
    pub fn name(&self) -> &'static str {
        match self {
            Self::AxiomsOfIntuition => "Axioms of Intuition",
            Self::AnticipationsOfPerception => "Anticipations of Perception",
            Self::AnalogiesOfExperience => "Analogies of Experience",
            Self::PostulatesOfEmpiricalThought => "Postulates of Empirical Thought",
        }
    }

    pub fn formula(&self) -> &'static str {
        match self {
            Self::AxiomsOfIntuition =>
                "All intuitions are extensive magnitudes",
            Self::AnticipationsOfPerception =>
                "In all appearances, the real that is an object of sensation has intensive magnitude, that is, a degree",
            Self::AnalogiesOfExperience =>
                "Experience is possible only through the representation of a necessary connection of perceptions",
            Self::PostulatesOfEmpiricalThought =>
                "What agrees with the formal conditions of experience (as to intuition and concepts) is possible; \
                 what is connected with the material conditions of experience (sensation) is actual; \
                 that which in its connection with the actual is determined by universal conditions of experience is necessary",
        }
    }

    pub fn group(&self) -> PrincipleGroup {
        match self {
            Self::AxiomsOfIntuition | Self::AnticipationsOfPerception => {
                PrincipleGroup::Mathematical
            }
            Self::AnalogiesOfExperience | Self::PostulatesOfEmpiricalThought => {
                PrincipleGroup::Dynamical
            }
        }
    }

    pub fn all() -> [Principle; 4] {
        [
            Self::AxiomsOfIntuition,
            Self::AnticipationsOfPerception,
            Self::AnalogiesOfExperience,
            Self::PostulatesOfEmpiricalThought,
        ]
    }
}

/// Mathematical principles concern magnitude (quantity and quality).
/// Dynamical principles concern existence (relation and modality).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrincipleGroup {
    Mathematical,
    Dynamical,
}

// ─── PrincipleCheck ──────────────────────────────────────────────────────────

/// The result of checking a proposition against a Principle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrincipleCheckResult {
    pub principle: Principle,
    pub holds: bool,
    pub note: String,
}

// ─── PrincipleSet ────────────────────────────────────────────────────────────

/// The complete set of Principles of Pure Understanding.
pub struct PrincipleSet;

impl PrincipleSet {
    /// Check all four principles against a set of propositions.
    pub fn check_all(propositions: &[Proposition]) -> Vec<super::PrincipleCheck> {
        Principle::all()
            .iter()
            .map(|&principle| Self::check(principle, propositions))
            .collect()
    }

    /// Check a single principle against propositions.
    pub fn check(principle: Principle, propositions: &[Proposition]) -> super::PrincipleCheck {
        let (holds, note) = match principle {
            Principle::AxiomsOfIntuition => {
                // Axioms: check that propositions describe things with spatial/temporal extension
                let spatial_words = [
                    "long", "wide", "size", "large", "small", "area", "volume", "extends",
                ];
                let has_extensive = propositions.iter().any(|p| {
                    let text = p.text.to_lowercase();
                    spatial_words.iter().any(|&w| text.contains(w))
                });
                (
                    // Mathematical principles are constitutive — they always hold for appearances
                    true,
                    if has_extensive {
                        "Extensive magnitude expressed explicitly".to_string()
                    } else {
                        "Extensive magnitude is presupposed for all spatiotemporal appearances"
                            .to_string()
                    },
                )
            }
            Principle::AnticipationsOfPerception => {
                // Anticipations: check for intensive magnitude / degree expressions
                let degree_words = [
                    "very",
                    "quite",
                    "extremely",
                    "slightly",
                    "somewhat",
                    "degrees",
                    "intensity",
                    "level",
                    "amount",
                    "barely",
                ];
                let has_degree = propositions.iter().any(|p| {
                    let text = p.text.to_lowercase();
                    degree_words.iter().any(|&w| text.contains(w))
                });
                (
                    true,
                    if has_degree {
                        "Intensive magnitude (degree) explicitly present".to_string()
                    } else {
                        "Intensive magnitude is presupposed for all sensory content".to_string()
                    },
                )
            }
            Principle::AnalogiesOfExperience => {
                // Analogies: check for causal/substance/community relations
                let causal_words = [
                    "because",
                    "therefore",
                    "causes",
                    "results in",
                    "follows from",
                    "leads to",
                    "if",
                    "then",
                ];
                let substance_words = [
                    "remains",
                    "persists",
                    "unchanged",
                    "permanent",
                    "underlying",
                    "continues",
                ];
                let has_analogy = propositions.iter().any(|p| {
                    let text = p.text.to_lowercase();
                    causal_words.iter().any(|&w| text.contains(w))
                        || substance_words.iter().any(|&w| text.contains(w))
                });
                (
                    has_analogy || !propositions.is_empty(),
                    if has_analogy {
                        "Causal or substance relations explicitly expressed".to_string()
                    } else {
                        "Analogies of Experience presupposed for any connected experience"
                            .to_string()
                    },
                )
            }
            Principle::PostulatesOfEmpiricalThought => {
                // Postulates: check for modality expressions
                let possibility_words = ["possible", "could", "might", "may", "can"];
                let actuality_words = ["actually", "in fact", "exists", "real", "indeed"];
                let necessity_words = ["necessarily", "must", "always", "inevitable", "certain"];
                let has_modality = propositions.iter().any(|p| {
                    let text = p.text.to_lowercase();
                    possibility_words.iter().any(|&w| text.contains(w))
                        || actuality_words.iter().any(|&w| text.contains(w))
                        || necessity_words.iter().any(|&w| text.contains(w))
                });
                (
                    true,
                    if has_modality {
                        "Explicit modal claims found (possibility/actuality/necessity)".to_string()
                    } else {
                        "Postulates of Empirical Thought apply as general conditions of experience"
                            .to_string()
                    },
                )
            }
        };

        super::PrincipleCheck {
            principle,
            holds,
            note: Some(note),
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
    fn all_principles_exist() {
        assert_eq!(Principle::all().len(), 4);
    }

    #[test]
    fn axioms_always_hold() {
        let props = vec![prop("The cat is on the mat")];
        let check = PrincipleSet::check(Principle::AxiomsOfIntuition, &props);
        assert!(check.holds);
    }

    #[test]
    fn analogies_detected_in_causal_text() {
        let props = vec![prop(
            "Water boils because heat causes the temperature to rise",
        )];
        let check = PrincipleSet::check(Principle::AnalogiesOfExperience, &props);
        assert!(check.holds);
    }

    #[test]
    fn mathematical_vs_dynamical() {
        assert_eq!(
            Principle::AxiomsOfIntuition.group(),
            PrincipleGroup::Mathematical
        );
        assert_eq!(
            Principle::AnalogiesOfExperience.group(),
            PrincipleGroup::Dynamical
        );
    }
}
