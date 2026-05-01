//! # Wittgensteinian Layer
//!
//! This module provides practical language-handling tools inspired by both
//! the early and late Wittgenstein, complementing Kant's formal system.
//!
//! ## Two Wittgensteins
//!
//! ### Early Wittgenstein (Tractatus Logico-Philosophicus, 1921)
//! - Propositions as pictures of facts
//! - The world is the totality of facts, not things
//! - Elementary propositions / atomic facts
//! - Truth-functions as logical composition
//! - The distinction between what can be said and what can only be shown
//!
//! ### Late Wittgenstein (Philosophical Investigations, 1953)
//! - Language games: "The meaning of a word is its use in the language"
//! - Forms of life: background practices that ground language games
//! - Rule-following: you cannot follow a rule privately
//! - Family resemblance: categories share overlapping features, not essences
//! - Meaning as use: context determines meaning, not reference

pub mod family_resemblance;
pub mod language_games;
pub mod rule_following;
pub mod tractatus;

pub use family_resemblance::{FamilyResemblance, Feature, ResemblanceScore};
pub use language_games::{FormOfLife, GameAnalysis, GameDetector, LanguageGame};
pub use rule_following::{Rule, RuleFollowingValidator, RuleReport};
pub use tractatus::{AtomicFact, FactPicture, LogicalSpace, Speakable};

use crate::types::Proposition;
use serde::{Deserialize, Serialize};

// ─── WittgensteinLayer ───────────────────────────────────────────────────────

/// The Wittgensteinian practical layer as a unified faculty.
#[derive(Default)]
pub struct WittgensteinLayer {
    pub game_detector: GameDetector,
    pub rule_validator: RuleFollowingValidator,
    pub family_resemblance: FamilyResemblance,
}

/// The output of the Wittgenstein layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WittgensteinReport {
    pub game_analysis: GameAnalysis,
    pub rule_reports: Vec<RuleReport>,
    pub fact_picture: Option<FactPicture>,
    pub speakable_boundary: SpeakableBoundary,
}

/// What can be said vs what can only be shown (Tractatus 7).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakableBoundary {
    /// Claims that can be expressed meaningfully in propositions.
    pub speakable: Vec<String>,
    /// What can only be shown, not said (logical form, ethical value, etc.).
    pub showable_only: Vec<String>,
}

impl WittgensteinLayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply(&self, propositions: &[Proposition]) -> WittgensteinReport {
        let text_combined: String = propositions
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let game_analysis = self.game_detector.analyze(&text_combined);
        let rule_reports = self.rule_validator.validate(propositions);
        let fact_picture = FactPicture::from_propositions(propositions);
        let speakable_boundary = SpeakableBoundary::analyze(propositions);

        WittgensteinReport {
            game_analysis,
            rule_reports,
            fact_picture: Some(fact_picture),
            speakable_boundary,
        }
    }
}

impl SpeakableBoundary {
    pub fn analyze(propositions: &[Proposition]) -> Self {
        let unspeakable_patterns = [
            "the meaning of life",
            "what is good in itself",
            "the value of",
            "what ought ultimately",
            "the logical form of language",
            "the nature of consciousness itself",
        ];

        let mut speakable = Vec::new();
        let mut showable_only = Vec::new();

        for prop in propositions {
            let text = prop.text.to_lowercase();
            if unspeakable_patterns.iter().any(|&p| text.contains(p)) {
                showable_only.push(prop.text.clone());
            } else {
                speakable.push(prop.text.clone());
            }
        }

        Self {
            speakable,
            showable_only,
        }
    }
}
