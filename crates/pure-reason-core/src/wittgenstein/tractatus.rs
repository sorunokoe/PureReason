//! # Tractatus Logico-Philosophicus Layer
//!
//! Implements key concepts from Wittgenstein's early work:
//! - The world is the totality of facts, not things (TLP 1.1)
//! - A proposition is a picture of a fact (TLP 2.1)
//! - Elementary propositions consist of names in combination (TLP 4.22)
//! - "Whereof one cannot speak, thereof one must be silent" (TLP 7)

use crate::types::Proposition;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── AtomicFact ──────────────────────────────────────────────────────────────

/// An atomic fact (Sachverhalt) — the simplest combination of objects.
/// "A state of affairs is a combination of objects." — TLP 2.01
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicFact {
    pub id: Uuid,
    /// The objects (names) that constitute this fact.
    pub objects: Vec<String>,
    /// The configuration (how the objects are combined).
    pub configuration: String,
    /// Whether this fact actually obtains (is true).
    pub obtains: Option<bool>,
}

impl AtomicFact {
    pub fn new(objects: Vec<String>, configuration: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            objects,
            configuration: configuration.into(),
            obtains: None,
        }
    }
}

// ─── LogicalSpace ─────────────────────────────────────────────────────────────

/// Logical space — the space of all possible facts.
/// "The facts in logical space are the world." — TLP 1.13
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalSpace {
    /// The set of possible atomic facts.
    pub possible_facts: Vec<AtomicFact>,
    /// Total number of possible combinations.
    pub dimensionality: usize,
}

impl LogicalSpace {
    pub fn from_objects(objects: &[String]) -> Self {
        // Each pair of objects defines a possible fact
        let mut possible_facts = Vec::new();
        for i in 0..objects.len() {
            for j in (i + 1)..objects.len() {
                possible_facts.push(AtomicFact::new(
                    vec![objects[i].clone(), objects[j].clone()],
                    format!("{} — {}", objects[i], objects[j]),
                ));
            }
        }
        let dimensionality = possible_facts.len();
        Self {
            possible_facts,
            dimensionality,
        }
    }
}

// ─── Speakable ───────────────────────────────────────────────────────────────

/// Whether a proposition is speakable (can be said) or only showable.
///
/// "What can be shown cannot be said." — TLP 4.1212
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Speakable {
    /// The proposition can be expressed in a meaningful proposition (picture of a fact).
    CanBeSaid,
    /// The proposition only shows something — its content cannot be stated.
    CanOnlyBeShown,
    /// Nonsense — neither says nor shows anything meaningful.
    Nonsense,
}

// ─── TruthFunction ────────────────────────────────────────────────────────────

/// A truth function combining atomic propositions.
/// "All propositions are truth-functions of elementary propositions." — TLP 5
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TruthFunction {
    Atomic(AtomicFact),
    Negation(Box<TruthFunction>),
    Conjunction(Box<TruthFunction>, Box<TruthFunction>),
    Disjunction(Box<TruthFunction>, Box<TruthFunction>),
    Conditional(Box<TruthFunction>, Box<TruthFunction>),
    Biconditional(Box<TruthFunction>, Box<TruthFunction>),
}

// ─── FactPicture ─────────────────────────────────────────────────────────────

/// A fact-picture — a proposition as a logical picture of a fact.
///
/// "A proposition is a picture of reality." — TLP 4.01
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactPicture {
    /// The propositions being pictured.
    pub propositions: Vec<Proposition>,
    /// The atomic facts extracted.
    pub atomic_facts: Vec<AtomicFact>,
    /// The logical space of all possible facts given the objects.
    pub logical_space: LogicalSpace,
    /// Whether each proposition is speakable.
    pub speakability: Vec<(String, Speakable)>,
}

impl FactPicture {
    /// Construct a FactPicture from a set of propositions.
    pub fn from_propositions(propositions: &[Proposition]) -> Self {
        let atomic_facts = Self::extract_atomic_facts(propositions);
        let objects: Vec<String> = atomic_facts
            .iter()
            .flat_map(|f| f.objects.iter().cloned())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let logical_space = LogicalSpace::from_objects(&objects);

        let speakability = propositions
            .iter()
            .map(|p| (p.text.clone(), Self::assess_speakability(p)))
            .collect();

        Self {
            propositions: propositions.to_vec(),
            atomic_facts,
            logical_space,
            speakability,
        }
    }

    fn extract_atomic_facts(propositions: &[Proposition]) -> Vec<AtomicFact> {
        // Extract subject-predicate pairs as atomic facts
        propositions
            .iter()
            .filter_map(|p| {
                if p.judgments.is_empty() {
                    // Treat the whole sentence as a fact with extracted nouns
                    let words: Vec<String> = p
                        .text
                        .split_whitespace()
                        .filter(|w| w.len() > 3)
                        .take(3)
                        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
                        .filter(|w| !w.is_empty())
                        .collect();
                    if words.len() >= 2 {
                        Some(AtomicFact::new(words, p.text.clone()))
                    } else {
                        None
                    }
                } else {
                    p.judgments.first().map(|j| {
                        AtomicFact::new(
                            vec![j.subject.text.clone(), j.predicate.text.clone()],
                            format!("{} {} {}", j.subject.text, "is", j.predicate.text),
                        )
                    })
                }
            })
            .collect()
    }

    /// Assess whether a proposition is speakable, showable-only, or nonsense.
    fn assess_speakability(proposition: &Proposition) -> Speakable {
        let text = proposition.text.to_lowercase();

        // Things that can only be shown (logical form, ethics, aesthetics)
        let showable_only_patterns = [
            "the logical form",
            "what is good in itself",
            "the meaning of life",
            "beauty itself",
            "value of values",
            "the ethical",
        ];

        // Genuine nonsense (self-referential paradoxes, category errors)
        let nonsense_patterns = [
            "this sentence is false",
            "colorless green ideas",
            "the present king of france",
            "nothing nothings",
            "being is",
        ];

        if nonsense_patterns.iter().any(|&p| text.contains(p)) {
            Speakable::Nonsense
        } else if showable_only_patterns.iter().any(|&p| text.contains(p)) {
            Speakable::CanOnlyBeShown
        } else {
            Speakable::CanBeSaid
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
    fn fact_picture_from_propositions() {
        let props = vec![
            prop("Water is a liquid at room temperature"),
            prop("Ice is a solid form of water"),
        ];
        let picture = FactPicture::from_propositions(&props);
        assert!(!picture.atomic_facts.is_empty());
    }

    #[test]
    fn speakability_assessment() {
        let p1 = prop("Water boils at 100 degrees");
        let p2 = prop("The logical form of language cannot be described");
        let pic = FactPicture::from_propositions(&[p1, p2]);
        let said = pic
            .speakability
            .iter()
            .filter(|(_, s)| *s == Speakable::CanBeSaid)
            .count();
        assert!(said >= 1);
    }

    #[test]
    fn logical_space_from_objects() {
        let objects: Vec<String> =
            vec!["water".to_string(), "ice".to_string(), "steam".to_string()];
        let space = LogicalSpace::from_objects(&objects);
        // C(3,2) = 3 pairs
        assert_eq!(space.possible_facts.len(), 3);
    }
}
