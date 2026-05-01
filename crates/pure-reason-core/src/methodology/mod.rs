//! # Transcendental Methodology
//!
//! The Transcendental Methodology is the fourth main part of the Critique of Pure Reason.
//! It contains Kant's reflection on the method, limits, and legitimate use of Pure Reason.
//!
//! ## Structure
//!
//! - [`discipline`] — Discipline of Pure Reason: constraints preventing dogmatic overreach
//! - [`canon`] — Canon of Pure Reason: the legitimate practical use of reason
//! - [`architectonic`] — Architectonic: the system as a whole, organized by ideas

pub mod architectonic;
pub mod canon;
pub mod discipline;

pub use architectonic::{Architectonic, SystemDescription};
pub use canon::{Canon, PracticalUse};
pub use discipline::{DisciplinaryRule, Discipline, ViolationReport};

use crate::types::Proposition;
use serde::{Deserialize, Serialize};

// ─── MethodologyLayer ────────────────────────────────────────────────────────

/// The Transcendental Methodology as a unified faculty.
#[derive(Default)]
pub struct MethodologyLayer {
    pub discipline: Discipline,
    pub canon: Canon,
    pub architectonic: Architectonic,
}

/// The output of the Methodology layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodologyReport {
    pub discipline_violations: Vec<ViolationReport>,
    pub legitimate_uses: Vec<PracticalUse>,
    pub system_description: SystemDescription,
}

impl MethodologyLayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply(&self, propositions: &[Proposition]) -> MethodologyReport {
        MethodologyReport {
            discipline_violations: self.discipline.check(propositions),
            legitimate_uses: self.canon.identify_legitimate_uses(propositions),
            system_description: self.architectonic.describe(),
        }
    }
}

impl MethodologyReport {
    /// Empty report for adaptive activation (TRIZ S-3): skipped when pre_score is low.
    pub fn empty() -> Self {
        Self {
            discipline_violations: Vec::new(),
            legitimate_uses: Vec::new(),
            system_description: SystemDescription {
                title: String::new(),
                unifying_idea: String::new(),
                components: Vec::new(),
                purpose: String::new(),
                limits: String::new(),
            },
        }
    }
}
