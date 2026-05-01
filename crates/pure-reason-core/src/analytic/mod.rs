//! # Transcendental Analytic
//!
//! The Transcendental Analytic is the second and largest part of the Critique of
//! Pure Reason. It analyses the pure concepts of the understanding (Categories)
//! and establishes the principles by which they apply to experience.
//!
//! ## Structure
//!
//! - [`categories`] — The 12 pure categories of the understanding
//! - [`judgments`] — The 12 logical forms of judgment (Table of Judgments)
//! - [`schematism`] — The temporal schemas that bridge categories to intuition
//! - [`principles`] — The Principles of Pure Understanding

pub mod categories;
pub mod judgments;
pub mod principles;
pub mod schematism;

pub use categories::{Category, CategoryAnalysis, CategoryApplication, CategoryGroup};
pub use judgments::{JudgmentAnalysis, JudgmentDetector};
pub use principles::{Principle, PrincipleSet};
pub use schematism::{Schema, Schematism, TemporalDetermination};

use crate::wittgenstein::language_games::FormOfLife;
use crate::{aesthetic::Intuition, error::Result};
use serde::{Deserialize, Serialize};

// ─── AnalyticLayer ───────────────────────────────────────────────────────────

/// The Transcendental Analytic as a unified faculty.
///
/// Takes an Intuition from the Aesthetic and applies:
/// 1. Category detection and application
/// 2. Schematism (temporal determination)
/// 3. Principle checking
#[derive(Default)]
pub struct AnalyticLayer {
    pub schematism: Schematism,
}

/// The output of the Analytic layer — a conceptually structured understanding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Understanding {
    /// Which categories were detected in the intuition.
    pub category_analysis: CategoryAnalysis,
    /// Judgment forms detected in the propositions.
    pub judgment_analysis: JudgmentAnalysis,
    /// Schematic (temporal) determinations for each applied category.
    pub schemas: Vec<Schema>,
    /// Which principles were checked, and whether they hold.
    pub principle_checks: Vec<PrincipleCheck>,
}

/// Result of checking a Principle of Pure Understanding against propositions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrincipleCheck {
    pub principle: Principle,
    pub holds: bool,
    pub note: Option<String>,
}

impl AnalyticLayer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply the full analytic to an intuition, producing an Understanding.
    pub fn apply(&self, intuition: &Intuition) -> Result<Understanding> {
        self.apply_with_game(intuition, None)
    }

    /// Apply the full analytic with optional language-game context (TRIZ S-1).
    ///
    /// When a game is provided, category scores are weighted by the game-context
    /// matrix — producing differentiated, context-sensitive scores at zero cost.
    pub fn apply_with_game(
        &self,
        intuition: &Intuition,
        game: Option<FormOfLife>,
    ) -> Result<Understanding> {
        let propositions = intuition.propositions();

        let mut category_analysis = match game {
            Some(g) => CategoryAnalysis::from_propositions_with_game(&propositions, g),
            None => CategoryAnalysis::from_propositions(&propositions),
        };
        let judgment_analysis = JudgmentAnalysis::from_propositions(&propositions);
        // S-8: Apply Judgment×Category joint boost — multiplies category scores
        // by form-specific factors (e.g. Universal+Causality → scientific law boost)
        category_analysis.apply_judgment_boost(&judgment_analysis);
        let schemas = self
            .schematism
            .apply_to_analysis(&category_analysis, &intuition.time);
        let principle_checks = PrincipleSet::check_all(&propositions);

        Ok(Understanding {
            category_analysis,
            judgment_analysis,
            schemas,
            principle_checks,
        })
    }
}
