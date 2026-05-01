//! # Family Resemblance
//!
//! Wittgenstein's concept of **family resemblance** (Familienähnlichkeit) challenges
//! the classical view that all members of a category share a single set of defining features.
//!
//! Instead, members of a category are held together by overlapping and criss-crossing
//! similarities — like members of a family who share various features (eyes, gait, build)
//! without any one feature being common to all.
//!
//! "I can think of no better expression to characterize these similarities
//! than 'family resemblances.'" — PI §67
//!
//! This module provides fuzzy categorization based on overlapping feature sets,
//! complementing the strict categorical analysis of the Kantian system.

use crate::types::Proposition;
use serde::{Deserialize, Serialize};

// ─── Feature ─────────────────────────────────────────────────────────────────

/// A feature that can be shared between instances.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub signal_terms: Vec<String>,
}

impl Feature {
    pub fn new(name: impl Into<String>, signal_terms: Vec<&str>) -> Self {
        Self {
            name: name.into(),
            signal_terms: signal_terms.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn matches(&self, text: &str) -> bool {
        let lower = text.to_lowercase();
        self.signal_terms.iter().any(|t| lower.contains(t.as_str()))
    }
}

// ─── FamilyCategory ──────────────────────────────────────────────────────────

/// A category defined by a family of overlapping features (not necessary conditions).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyCategory {
    pub name: String,
    pub features: Vec<Feature>,
    /// Minimum number of features required for membership.
    pub membership_threshold: usize,
}

impl FamilyCategory {
    pub fn new(name: impl Into<String>, features: Vec<Feature>, threshold: usize) -> Self {
        Self {
            name: name.into(),
            features,
            membership_threshold: threshold,
        }
    }

    /// How many features does a text instance share with this category?
    pub fn feature_overlap(&self, text: &str) -> Vec<&Feature> {
        self.features.iter().filter(|f| f.matches(text)).collect()
    }

    /// Is this text a member of this category (above threshold)?
    pub fn is_member(&self, text: &str) -> bool {
        self.feature_overlap(text).len() >= self.membership_threshold
    }

    /// Resemblance score: fraction of features matched.
    pub fn resemblance_score(&self, text: &str) -> f64 {
        if self.features.is_empty() {
            return 0.0;
        }
        self.feature_overlap(text).len() as f64 / self.features.len() as f64
    }
}

// ─── ResemblanceScore ────────────────────────────────────────────────────────

/// A resemblance score between a text and a family category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResemblanceScore {
    pub category: String,
    pub score: f64,
    pub matched_features: Vec<String>,
    pub is_member: bool,
}

// ─── FamilyResemblance ───────────────────────────────────────────────────────

/// The family resemblance categorizer — fuzzy categorization based on overlapping features.
pub struct FamilyResemblance {
    pub categories: Vec<FamilyCategory>,
}

impl FamilyResemblance {
    pub fn new() -> Self {
        Self {
            categories: Self::default_categories(),
        }
    }

    /// Categorize a proposition using family resemblance.
    pub fn categorize(&self, proposition: &Proposition) -> Vec<ResemblanceScore> {
        self.categories
            .iter()
            .map(|cat| {
                let overlapping = cat.feature_overlap(&proposition.text);
                ResemblanceScore {
                    category: cat.name.clone(),
                    score: cat.resemblance_score(&proposition.text),
                    matched_features: overlapping.iter().map(|f| f.name.clone()).collect(),
                    is_member: cat.is_member(&proposition.text),
                }
            })
            .filter(|s| s.score > 0.0)
            .collect()
    }

    /// The default set of family categories for reasoning about text.
    fn default_categories() -> Vec<FamilyCategory> {
        vec![
            FamilyCategory::new(
                "Game (Wittgenstein's example)",
                vec![
                    Feature::new(
                        "competition",
                        vec!["win", "lose", "compete", "score", "player"],
                    ),
                    Feature::new(
                        "rules",
                        vec!["rules", "moves", "allowed", "permitted", "forbidden"],
                    ),
                    Feature::new("fun", vec!["fun", "play", "enjoy", "amuse", "entertain"]),
                    Feature::new(
                        "skill",
                        vec!["skill", "strategy", "tactic", "practice", "technique"],
                    ),
                    Feature::new(
                        "board/equipment",
                        vec!["board", "pieces", "cards", "ball", "field"],
                    ),
                ],
                2, // Need at least 2 features for membership
            ),
            FamilyCategory::new(
                "Causal Claim",
                vec![
                    Feature::new(
                        "causation word",
                        vec!["cause", "because", "therefore", "leads to", "results in"],
                    ),
                    Feature::new(
                        "temporal succession",
                        vec!["then", "after", "subsequently", "following"],
                    ),
                    Feature::new(
                        "necessity",
                        vec!["necessarily", "must", "inevitably", "always"],
                    ),
                    Feature::new(
                        "mechanism",
                        vec!["mechanism", "process", "chain", "pathway"],
                    ),
                ],
                1,
            ),
            FamilyCategory::new(
                "Normative Claim",
                vec![
                    Feature::new(
                        "deontic",
                        vec!["ought", "should", "must", "duty", "obligation"],
                    ),
                    Feature::new(
                        "value",
                        vec!["good", "bad", "right", "wrong", "better", "worse"],
                    ),
                    Feature::new(
                        "recommendation",
                        vec!["recommend", "advise", "suggest", "urge"],
                    ),
                    Feature::new(
                        "criticism",
                        vec!["criticize", "wrong to", "mistaken", "incorrect"],
                    ),
                ],
                1,
            ),
            FamilyCategory::new(
                "Epistemic Claim",
                vec![
                    Feature::new(
                        "knowledge",
                        vec!["know", "knowledge", "understand", "certain", "sure"],
                    ),
                    Feature::new(
                        "belief",
                        vec!["believe", "think", "suppose", "assume", "conjecture"],
                    ),
                    Feature::new(
                        "evidence",
                        vec!["evidence", "proof", "reason", "data", "observation"],
                    ),
                    Feature::new(
                        "doubt",
                        vec!["doubt", "uncertain", "unclear", "unknown", "possibly"],
                    ),
                ],
                1,
            ),
        ]
    }
}

impl Default for FamilyResemblance {
    fn default() -> Self {
        Self::new()
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
    fn causal_claim_categorized() {
        let p = prop("Water boils because heat causes the temperature to rise");
        let fr = FamilyResemblance::new();
        let scores = fr.categorize(&p);
        let causal = scores.iter().find(|s| s.category.contains("Causal"));
        assert!(causal.is_some());
        assert!(causal.unwrap().score > 0.0);
    }

    #[test]
    fn normative_claim_categorized() {
        let p = prop("We ought to treat people fairly and do what is right and good");
        let fr = FamilyResemblance::new();
        let scores = fr.categorize(&p);
        let normative = scores.iter().find(|s| s.category.contains("Normative"));
        assert!(normative.is_some());
    }

    #[test]
    fn game_categorized() {
        let p = prop(
            "Chess is a game where players compete using skill and strategy with pieces on a board",
        );
        let fr = FamilyResemblance::new();
        let scores = fr.categorize(&p);
        let game = scores.iter().find(|s| s.category.contains("Game"));
        assert!(game.is_some());
        assert!(game.unwrap().is_member);
    }
}
