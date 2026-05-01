//! # Transcendental Ideas
//!
//! The Transcendental Ideas are the pure concepts of Reason (not Understanding).
//! Reason seeks the unconditioned — the complete totality of conditions.
//!
//! There are exactly three Transcendental Ideas, each corresponding to one type
//! of syllogistic inference:
//!
//! 1. **The Psychological Idea (Soul)** — totality of the thinking subject
//! 2. **The Cosmological Idea (World)** — totality of conditions of appearances
//! 3. **The Theological Idea (God)** — totality of all conditions of thought
//!
//! These ideas are **regulative**: they guide inquiry by positing an ideal of
//! completeness, but they do NOT constitute real objects. To treat them as
//! constitutive leads to dialectical illusion.

use serde::{Deserialize, Serialize};

// ─── PsychologicalIdea ───────────────────────────────────────────────────────

/// The Psychological Idea — the soul as the unconditioned unity of the thinking subject.
///
/// "Soul" here means the transcendental subject, not necessarily a spiritual entity.
/// Kant shows that any attempt to determine the soul's properties by pure reason
/// leads to the Paralogisms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PsychologicalIdea {
    /// The soul as substance — permanent underlying subject.
    Substantiality,
    /// The soul as simple — without parts.
    Simplicity,
    /// The soul as numerically identical across time — personal identity.
    Personality,
    /// The soul's relation to possible objects in space — ideality.
    Ideality,
}

// ─── CosmologicalIdea ────────────────────────────────────────────────────────

/// The Cosmological Idea — the World as the unconditioned totality of appearances.
///
/// Attempts to determine the World-totality lead to the four Antinomies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CosmologicalIdea {
    /// The infinity or finitude of the world in time and space.
    Infinity,
    /// Whether composites consist of simples.
    Simplicity,
    /// Freedom vs. natural causality.
    Freedom,
    /// Whether a necessary being exists.
    Necessity,
}

// ─── TheologicalIdea ─────────────────────────────────────────────────────────

/// The Theological Idea — God as the unconditioned ground of all conditions.
///
/// Kant analyzes three "proofs" of God's existence and shows they all fail:
/// - Ontological argument (from the concept of a perfect being)
/// - Cosmological argument (from contingent existence)
/// - Physico-theological argument (from design in nature)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TheologicalIdea {
    /// God as the ens realissimum — most real being.
    MostRealBeing,
    /// God as the necessary being — that which must exist.
    NecessaryBeing,
    /// God as the first cause — the uncaused cause.
    FirstCause,
    /// God as the designer of nature — from design argument.
    Designer,
}

// ─── IdeaUse ─────────────────────────────────────────────────────────────────

/// Whether a Transcendental Idea is being used legitimately or not.
///
/// This is the crucial Kantian distinction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdeaUse {
    /// Regulative use: the Idea serves as a goal or ideal to guide inquiry.
    /// This is the ONLY legitimate use of Transcendental Ideas.
    Regulative,
    /// Constitutive use: the Idea is treated as denoting a real object.
    /// This leads to dialectical illusion. This use is ILLEGITIMATE.
    Constitutive,
}

impl IdeaUse {
    pub fn is_legitimate(&self) -> bool {
        *self == IdeaUse::Regulative
    }
}

// ─── TranscendentalIdea ──────────────────────────────────────────────────────

/// One of the three Transcendental Ideas of Pure Reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TranscendentalIdea {
    /// The Soul — unconditioned unity of the thinking subject.
    Soul(PsychologicalIdea),
    /// The World — unconditioned totality of appearances.
    World(CosmologicalIdea),
    /// God — unconditioned ground of all conditions.
    God(TheologicalIdea),
}

impl TranscendentalIdea {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Soul(_) => "Soul (Psychological Idea)",
            Self::World(_) => "World (Cosmological Idea)",
            Self::God(_) => "God (Theological Idea)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Soul(_) => "The unconditioned unity of the thinking subject",
            Self::World(_) => "The unconditioned totality of all appearances",
            Self::God(_) => "The unconditioned ground of all conditions of thought",
        }
    }

    /// The legitimate use of this Idea is always regulative, never constitutive.
    pub fn legitimate_use(&self) -> IdeaUse {
        IdeaUse::Regulative
    }

    /// All concrete sub-variants.
    pub fn all_souls() -> [TranscendentalIdea; 4] {
        [
            Self::Soul(PsychologicalIdea::Substantiality),
            Self::Soul(PsychologicalIdea::Simplicity),
            Self::Soul(PsychologicalIdea::Personality),
            Self::Soul(PsychologicalIdea::Ideality),
        ]
    }

    pub fn all_worlds() -> [TranscendentalIdea; 4] {
        [
            Self::World(CosmologicalIdea::Infinity),
            Self::World(CosmologicalIdea::Simplicity),
            Self::World(CosmologicalIdea::Freedom),
            Self::World(CosmologicalIdea::Necessity),
        ]
    }

    pub fn all_gods() -> [TranscendentalIdea; 4] {
        [
            Self::God(TheologicalIdea::MostRealBeing),
            Self::God(TheologicalIdea::NecessaryBeing),
            Self::God(TheologicalIdea::FirstCause),
            Self::God(TheologicalIdea::Designer),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regulative_use_is_legitimate() {
        assert!(IdeaUse::Regulative.is_legitimate());
        assert!(!IdeaUse::Constitutive.is_legitimate());
    }

    #[test]
    fn ideas_have_names() {
        let soul = TranscendentalIdea::Soul(PsychologicalIdea::Substantiality);
        let world = TranscendentalIdea::World(CosmologicalIdea::Infinity);
        let god = TranscendentalIdea::God(TheologicalIdea::NecessaryBeing);

        assert!(!soul.name().is_empty());
        assert!(!world.name().is_empty());
        assert!(!god.name().is_empty());
    }
}
