//! Core primitive types shared across all modules.
//!
//! These are the fundamental building blocks that appear throughout Kant's system:
//! terms, propositions, judgments, and the faculty trait.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Term ────────────────────────────────────────────────────────────────────

/// A term is the basic unit of meaning: a subject or predicate in a judgment.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Term {
    pub id: Uuid,
    /// The surface-level linguistic representation.
    pub text: String,
    /// Optional semantic annotations.
    pub annotations: Vec<String>,
}

impl Term {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            text: text.into(),
            annotations: Vec::new(),
        }
    }

    pub fn with_annotation(mut self, annotation: impl Into<String>) -> Self {
        self.annotations.push(annotation.into());
        self
    }
}

// ─── Copula ──────────────────────────────────────────────────────────────────

/// The linking verb connecting subject to predicate in a judgment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Copula {
    /// "S is P"
    Is,
    /// "S is not P"
    IsNot,
    /// "S can be P" (problematic)
    CanBe,
    /// "S must be P" (apodeictic)
    MustBe,
}

// ─── JudgmentForm ─────────────────────────────────────────────────────────────

/// Kant's Table of Judgments — 12 logical forms of judgment.
///
/// These are the logical forms from which the 12 Categories are derived.
/// Organized in four groups of three.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JudgmentForm {
    // Quantity
    /// "All S are P"
    Universal,
    /// "Some S are P"
    Particular,
    /// "This S is P"
    Singular,

    // Quality
    /// "S is P"
    Affirmative,
    /// "S is not P"
    Negative,
    /// "S is non-P" (infinite judgment)
    Infinite,

    // Relation
    /// "If A then B" — hypothetical
    Hypothetical,
    /// "Either A or B" — disjunctive
    Disjunctive,
    /// "S is P" — categorical
    Categorical,

    // Modality
    /// "S may be P" — possible
    Problematic,
    /// "S is P" — actual
    Assertoric,
    /// "S must be P" — necessary
    Apodeictic,
}

impl JudgmentForm {
    pub fn group(&self) -> JudgmentGroup {
        match self {
            Self::Universal | Self::Particular | Self::Singular => JudgmentGroup::Quantity,
            Self::Affirmative | Self::Negative | Self::Infinite => JudgmentGroup::Quality,
            Self::Categorical | Self::Hypothetical | Self::Disjunctive => JudgmentGroup::Relation,
            Self::Problematic | Self::Assertoric | Self::Apodeictic => JudgmentGroup::Modality,
        }
    }

    /// All 12 judgment forms.
    pub fn all() -> [JudgmentForm; 12] {
        [
            Self::Universal,
            Self::Particular,
            Self::Singular,
            Self::Affirmative,
            Self::Negative,
            Self::Infinite,
            Self::Categorical,
            Self::Hypothetical,
            Self::Disjunctive,
            Self::Problematic,
            Self::Assertoric,
            Self::Apodeictic,
        ]
    }
}

/// The four groups of judgment forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JudgmentGroup {
    Quantity,
    Quality,
    Relation,
    Modality,
}

// ─── Judgment ────────────────────────────────────────────────────────────────

/// A judgment is the basic unit of thought for Kant: a relation between concepts.
///
/// "All knowledge is expressed in judgments." — Kant, CPR B94
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Judgment {
    pub id: Uuid,
    pub subject: Term,
    pub predicate: Term,
    pub copula: Copula,
    pub form: JudgmentForm,
    /// Source text from which this judgment was extracted.
    pub source: Option<String>,
}

impl Judgment {
    pub fn new(subject: Term, predicate: Term, copula: Copula, form: JudgmentForm) -> Self {
        Self {
            id: Uuid::new_v4(),
            subject,
            predicate,
            copula,
            form,
            source: None,
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

// ─── Proposition ─────────────────────────────────────────────────────────────

/// A proposition is a complete meaningful sentence that can be true or false.
///
/// Distinct from a Judgment in that propositions may be compound and
/// need not follow the subject-copula-predicate structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposition {
    pub id: Uuid,
    pub text: String,
    /// The judgment(s) this proposition encodes.
    pub judgments: Vec<Judgment>,
    pub kind: PropositionKind,
}

/// The epistemological kind of a proposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropositionKind {
    /// True by analysis of concepts alone (e.g., "All bachelors are unmarried").
    AnalyticApriori,
    /// Informative and learned from experience (e.g., "The sun rises in the east").
    SyntheticAposteriori,
    /// Informative yet necessarily true — Kant's central discovery (e.g., "7+5=12").
    SyntheticApriori,
    /// Unknown or not yet classified.
    Unknown,
}

impl Proposition {
    pub fn new(text: impl Into<String>, kind: PropositionKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            text: text.into(),
            judgments: Vec::new(),
            kind,
        }
    }
}

// ─── Faculty Trait ───────────────────────────────────────────────────────────

/// A cognitive faculty processes input and produces output.
///
/// Corresponds to Kant's faculties: Sensibility, Understanding, Reason.
pub trait Faculty {
    type Input;
    type Output;

    /// The name of this faculty.
    fn name(&self) -> &'static str;

    /// Apply the faculty to the given input.
    fn apply(&self, input: Self::Input) -> crate::Result<Self::Output>;
}

// ─── Synthesis Trait ─────────────────────────────────────────────────────────

/// Kant's synthesis: the act of combining representations.
///
/// Three forms of synthesis: apprehension (aesthetic), reproduction (imagination),
/// recognition (understanding).
pub trait Synthesize<A, B> {
    fn synthesize(&self, a: A, b: B) -> crate::Result<Proposition>;
}

// ─── Validate Trait ──────────────────────────────────────────────────────────

/// Validate a proposition or judgment against rules of the understanding or reason.
pub trait Validate {
    type Report;

    fn validate(&self, proposition: &Proposition) -> Self::Report;
}

// ─── Confidence ──────────────────────────────────────────────────────────────

/// A confidence score in [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Confidence(f64);

impl Confidence {
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    pub fn certain() -> Self {
        Self(1.0)
    }
    pub fn impossible() -> Self {
        Self(0.0)
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}

// ─── Epistemic Status ────────────────────────────────────────────────────────

/// Whether a claim is within or beyond possible experience.
///
/// Phenomena: things as they appear to us (within possible experience).
/// Noumena: things as they are in themselves (beyond possible experience).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpistemicStatus {
    /// Within possible experience — legitimate knowledge claim.
    Phenomenon,
    /// Beyond possible experience — only regulative use permitted.
    Noumenon,
    /// Unknown epistemic status.
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn term_creation() {
        let t = Term::new("water");
        assert_eq!(t.text, "water");
        assert!(t.annotations.is_empty());
    }

    #[test]
    fn judgment_form_groups() {
        assert_eq!(JudgmentForm::Universal.group(), JudgmentGroup::Quantity);
        assert_eq!(JudgmentForm::Affirmative.group(), JudgmentGroup::Quality);
        assert_eq!(JudgmentForm::Categorical.group(), JudgmentGroup::Relation);
        assert_eq!(JudgmentForm::Assertoric.group(), JudgmentGroup::Modality);
    }

    #[test]
    fn all_judgment_forms_are_twelve() {
        assert_eq!(JudgmentForm::all().len(), 12);
    }

    #[test]
    fn confidence_clamped() {
        assert_eq!(Confidence::new(1.5).value(), 1.0);
        assert_eq!(Confidence::new(-0.5).value(), 0.0);
    }
}
