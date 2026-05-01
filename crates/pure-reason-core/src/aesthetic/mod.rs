//! # Transcendental Aesthetic
//!
//! Kant's Transcendental Aesthetic is the first part of the Critique of Pure Reason.
//! It establishes that **Space** and **Time** are not properties of things-in-themselves
//! but pure forms of sensible intuition — the a priori conditions for any experience.
//!
//! In this system, the Aesthetic layer processes raw text input into a structured
//! `Intuition` — a spatiotemporal representation ready for the Understanding.

pub mod segmenter;
pub mod space;
pub mod structured;
pub mod time;

pub use segmenter::SegmentedInput;

use crate::{error::Result, types::Proposition};
use serde::{Deserialize, Serialize};

pub use space::{SpaceForm, SpatialRelation, StructuralNode};
pub use time::{TemporalEvent, TemporalOrder, TimeForm};

// ─── Manifold ────────────────────────────────────────────────────────────────

/// The raw, unorganized multiplicity of sensory data.
///
/// Kant calls this the "manifold of intuition" — the raw given before the forms
/// of intuition (Space and Time) have organized it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifold {
    /// The raw text input.
    pub raw: String,
    /// Tokens extracted from the raw text.
    pub tokens: Vec<String>,
    /// Sentences extracted from the raw text.
    pub sentences: Vec<String>,
}

impl Manifold {
    /// Create a Manifold from raw text by performing basic tokenization and segmentation.
    pub fn from_text(text: impl Into<String>) -> Self {
        let raw = text.into();
        let tokens = tokenize(&raw);
        let sentences = segment_sentences(&raw);
        Self {
            raw,
            tokens,
            sentences,
        }
    }
}

fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|s| {
            s.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

fn segment_sentences(text: &str) -> Vec<String> {
    text.split(['.', '!', '?'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

// ─── Intuition ───────────────────────────────────────────────────────────────

/// An intuition is the output of the Aesthetic faculty.
///
/// It is the manifold organized by the pure forms of Space and Time.
/// All subsequent cognitive processing (by the Understanding) operates on Intuitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intuition {
    /// The original manifold from which this intuition was formed.
    pub manifold: Manifold,
    /// The spatial form — structural/relational organization.
    pub space: SpaceForm,
    /// The temporal form — sequential/temporal organization.
    pub time: TimeForm,
}

impl Intuition {
    /// The core function of the Aesthetic: synthesize a Manifold into an Intuition.
    ///
    /// This applies both the Form of Space and the Form of Time to the raw manifold,
    /// producing a structured intuition ready for conceptual processing.
    pub fn from_manifold(manifold: Manifold) -> Result<Self> {
        let space = SpaceForm::organize(&manifold)?;
        let time = TimeForm::organize(&manifold)?;
        Ok(Self {
            manifold,
            space,
            time,
        })
    }

    /// Convenience: create an Intuition directly from text.
    pub fn from_text(text: impl Into<String>) -> Result<Self> {
        Self::from_manifold(Manifold::from_text(text))
    }

    /// The propositions encoded in this intuition's sentences.
    pub fn propositions(&self) -> Vec<Proposition> {
        self.manifold
            .sentences
            .iter()
            .map(|s| Proposition::new(s, crate::types::PropositionKind::Unknown))
            .collect()
    }
}

// ─── AestheticLayer ──────────────────────────────────────────────────────────

/// The Transcendental Aesthetic as a faculty.
pub struct AestheticLayer;

impl crate::types::Faculty for AestheticLayer {
    type Input = String;
    type Output = Intuition;

    fn name(&self) -> &'static str {
        "Transcendental Aesthetic"
    }

    fn apply(&self, input: String) -> Result<Intuition> {
        Intuition::from_text(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifold_tokenizes() {
        let m = Manifold::from_text("The cat sat on the mat.");
        assert!(m.tokens.contains(&"cat".to_string()));
        assert!(!m.tokens.is_empty());
    }

    #[test]
    fn manifold_segments_sentences() {
        let m = Manifold::from_text("Snow is white. The sky is blue.");
        assert_eq!(m.sentences.len(), 2);
    }

    #[test]
    fn intuition_from_text() {
        let i = Intuition::from_text("The sun rises every morning.").unwrap();
        assert!(!i.manifold.tokens.is_empty());
    }
}
