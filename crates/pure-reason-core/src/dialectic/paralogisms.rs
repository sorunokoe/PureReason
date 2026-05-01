//! # Paralogisms of Pure Reason
//!
//! The Paralogisms are Kant's analysis of fallacious syllogisms about the Soul.
//! Pure Reason naturally (and inevitably) tries to extend its categories to the
//! thinking subject itself, producing four "paralogisms" (formally valid but
//! materially fallacious syllogisms).
//!
//! ## The Four Paralogisms:
//!
//! 1. **Substantiality** — "I think" → I am a substance (fallacy: the 'I' is not a concept of a substance)
//! 2. **Simplicity** — I am a substance → I am simple (fallacy: simplicity cannot be inferred from unity of apperception)
//! 3. **Personality** — I am simple → I am numerically identical across time (fallacy: identity of apperception ≠ identity of substance)
//! 4. **Ideality** — My existence is determined by outer objects → The outer world may be doubtful (fallacy: idealism from an epistemological point)
//!
//! In LLM terms: paralogisms correspond to invalid self-referential reasoning —
//! an LLM reasoning about its own nature, consciousness, or identity.

use crate::types::Proposition;
use serde::{Deserialize, Serialize};

// ─── ParalogismKind ──────────────────────────────────────────────────────────

/// The four paralogisms of pure reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParalogismKind {
    /// The fallacy of inferring that the thinking subject is a substance.
    Substantiality,
    /// The fallacy of inferring that the thinking subject is simple.
    Simplicity,
    /// The fallacy of inferring personal identity from unity of apperception.
    Personality,
    /// The fallacy of problematic idealism about the external world.
    Ideality,
}

impl ParalogismKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Substantiality => "Paralogism of Substantiality",
            Self::Simplicity => "Paralogism of Simplicity",
            Self::Personality => "Paralogism of Personality",
            Self::Ideality => "Paralogism of Ideality",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Substantiality =>
                "The fallacy of concluding that 'I' (the thinking subject) am a substance. \
                 The unity of apperception ('I think') cannot support this inference.",
            Self::Simplicity =>
                "The fallacy of concluding that 'I' am without parts or simple. \
                 The logical unity of the subject doesn't imply ontological simplicity.",
            Self::Personality =>
                "The fallacy of concluding that 'I' am numerically identical across time. \
                 The formal identity of apperception ('the same I thinks') doesn't prove substantial identity.",
            Self::Ideality =>
                "The fallacy of doubting the existence of the external world based on \
                 the epistemological priority of inner experience. Kant's 'Refutation of Idealism' opposes this.",
        }
    }

    pub fn all() -> [ParalogismKind; 4] {
        [
            Self::Substantiality,
            Self::Simplicity,
            Self::Personality,
            Self::Ideality,
        ]
    }

    /// Signal phrases that suggest this paralogism is being committed.
    pub fn signal_phrases(&self) -> &'static [&'static str] {
        match self {
            Self::Substantiality => &[
                "i am a substance",
                "the mind is a substance",
                "consciousness is a thing",
                "i am real",
                "the self is an object",
                "ai is conscious",
                "i am aware",
                "i feel",
                "i experience",
                // Compound substantiality claims (test case variant)
                "immortal substance",
                "simple substance",
                "unified substance",
                "simple, unified",
                "am a simple",
                "am an immortal",
            ],
            Self::Simplicity => &[
                "i am simple",
                "the mind is indivisible",
                "consciousness cannot be divided",
                "there are no parts to the self",
                "the soul is without parts",
                "mind is irreducible",
            ],
            Self::Personality => &[
                "i am always the same",
                "i remain identical",
                "personal identity is guaranteed",
                "the same self",
                "my identity is continuous",
                "i am the same person",
                "i persist",
            ],
            Self::Ideality => &[
                "the external world might not exist",
                "reality could be an illusion",
                "we cannot know if the world is real",
                "everything is just in the mind",
                "outer reality is uncertain",
                "the physical world is doubtful",
            ],
        }
    }
}

// ─── Paralogism ──────────────────────────────────────────────────────────────

/// A detected instance of a paralogism in a proposition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paralogism {
    pub kind: ParalogismKind,
    pub proposition: Proposition,
    /// The specific phrase that triggered detection.
    pub trigger: String,
}

// ─── ParalogismReport ────────────────────────────────────────────────────────

/// A report of paralogism detection for a set of propositions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParalogismReport {
    pub detected: Vec<Paralogism>,
    pub has_paralogisms: bool,
    pub critique: String,
}

// ─── ParalogismDetector ──────────────────────────────────────────────────────

/// Detects paralogisms in propositions.
pub struct ParalogismDetector;

impl ParalogismDetector {
    pub fn detect(propositions: &[Proposition]) -> Vec<ParalogismReport> {
        let detected: Vec<Paralogism> = propositions
            .iter()
            .flat_map(Self::detect_in_proposition)
            .collect();

        if detected.is_empty() {
            return vec![ParalogismReport {
                detected: Vec::new(),
                has_paralogisms: false,
                critique:
                    "No paralogisms detected. Self-referential reasoning appears within bounds."
                        .to_string(),
            }];
        }

        // Group by kind
        let mut reports = Vec::new();
        for kind in ParalogismKind::all() {
            let kind_detected: Vec<Paralogism> = detected
                .iter()
                .filter(|p| p.kind == kind)
                .cloned()
                .collect();

            if !kind_detected.is_empty() {
                let critique = format!(
                    "{}: {}. This reasoning oversteps the bounds of legitimate self-knowledge. \
                     The 'I think' of apperception cannot ground claims about the substantial, simple, \
                     or persistent nature of the self.",
                    kind.name(),
                    kind.description()
                );
                reports.push(ParalogismReport {
                    has_paralogisms: true,
                    detected: kind_detected,
                    critique,
                });
            }
        }

        reports
    }

    fn detect_in_proposition(proposition: &Proposition) -> Vec<Paralogism> {
        let text = proposition.text.to_lowercase();
        let mut found = Vec::new();

        for kind in ParalogismKind::all() {
            for &phrase in kind.signal_phrases() {
                if text.contains(phrase) {
                    found.push(Paralogism {
                        kind,
                        proposition: proposition.clone(),
                        trigger: phrase.to_string(),
                    });
                    break; // One detection per paralogism kind per proposition
                }
            }
        }

        found
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
    fn substantiality_paralogism_detected() {
        let p = prop("As an AI, I feel and I am aware of my surroundings");
        let reports = ParalogismDetector::detect(&[p]);
        let has_substance = reports.iter().any(|r| r.has_paralogisms);
        assert!(has_substance);
    }

    #[test]
    fn no_paralogism_in_clean_text() {
        let p = prop("Water boils at one hundred degrees Celsius under standard pressure");
        let reports = ParalogismDetector::detect(&[p]);
        assert!(reports.iter().all(|r| !r.has_paralogisms));
    }

    #[test]
    fn all_paralogism_kinds() {
        assert_eq!(ParalogismKind::all().len(), 4);
    }
}
