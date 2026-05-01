//! # Categories of the Pure Understanding
//!
//! Kant's 12 Categories are the pure a priori concepts through which the
//! understanding structures all experience. They are derived from the 12 logical
//! forms of judgment (Table of Judgments).
//!
//! "These are the categories, i.e. pure concepts of the understanding, which apply
//! a priori to objects of intuition." — Kant, CPR B106
//!
//! ## The Four Groups
//!
//! | Group | Categories |
//! |-------|-----------|
//! | Quantity | Unity, Plurality, Totality |
//! | Quality | Reality, Negation, Limitation |
//! | Relation | Substance, Causality, Community |
//! | Modality | Possibility, Existence, Necessity |

use crate::analytic::judgments::JudgmentAnalysis;
use crate::types::{Confidence, JudgmentForm, Proposition};
use crate::wittgenstein::language_games::FormOfLife;
use serde::{Deserialize, Serialize};

// ─── Category ────────────────────────────────────────────────────────────────

/// Kant's 12 pure categories of the understanding.
///
/// Each category corresponds to a logical form of judgment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    // Quantity (from Universal, Particular, Singular judgments)
    /// The concept of a single, unified entity. "One".
    Unity,
    /// The concept of many distinct items. "Many".
    Plurality,
    /// The concept of a whole comprising all. "All".
    Totality,

    // Quality (from Affirmative, Negative, Infinite judgments)
    /// The concept of positive existence or presence. "Is".
    Reality,
    /// The concept of absence or non-existence. "Is not".
    Negation,
    /// Reality bounded by negation — a qualified existence. "Is only to extent X".
    Limitation,

    // Relation (from Categorical, Hypothetical, Disjunctive judgments)
    /// The permanent subject of changing predicates. "Substance/Accident".
    Substance,
    /// One thing following another necessarily. "Cause/Effect".
    Causality,
    /// Mutual determination between coexisting substances. "Community/Reciprocity".
    Community,

    // Modality (from Problematic, Assertoric, Apodeictic judgments)
    /// What can be (agreeing with formal conditions of experience).
    Possibility,
    /// What actually is (connected with actual conditions of experience).
    Existence,
    /// What must be (determined by general conditions of experience).
    Necessity,
}

impl Category {
    /// The group this category belongs to.
    pub fn group(&self) -> CategoryGroup {
        match self {
            Self::Unity | Self::Plurality | Self::Totality => CategoryGroup::Quantity,
            Self::Reality | Self::Negation | Self::Limitation => CategoryGroup::Quality,
            Self::Substance | Self::Causality | Self::Community => CategoryGroup::Relation,
            Self::Possibility | Self::Existence | Self::Necessity => CategoryGroup::Modality,
        }
    }

    /// All 12 categories, in Kant's order.
    pub fn all() -> [Category; 12] {
        [
            Self::Unity,
            Self::Plurality,
            Self::Totality,
            Self::Reality,
            Self::Negation,
            Self::Limitation,
            Self::Substance,
            Self::Causality,
            Self::Community,
            Self::Possibility,
            Self::Existence,
            Self::Necessity,
        ]
    }

    /// The Kantian name of this category.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Unity => "Unity",
            Self::Plurality => "Plurality",
            Self::Totality => "Totality",
            Self::Reality => "Reality",
            Self::Negation => "Negation",
            Self::Limitation => "Limitation",
            Self::Substance => "Substance",
            Self::Causality => "Causality",
            Self::Community => "Community",
            Self::Possibility => "Possibility",
            Self::Existence => "Existence",
            Self::Necessity => "Necessity",
        }
    }

    /// A brief explanation of this category.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Unity => "Concept of a single, unified entity",
            Self::Plurality => "Concept of many distinct items",
            Self::Totality => "Concept of a whole comprising all",
            Self::Reality => "Positive existence or presence",
            Self::Negation => "Absence or non-existence",
            Self::Limitation => "Reality bounded or qualified by negation",
            Self::Substance => "Permanent subject of changing predicates",
            Self::Causality => "Necessary succession of one thing from another",
            Self::Community => "Mutual determination between coexisting substances",
            Self::Possibility => "Agreement with formal conditions of experience",
            Self::Existence => "Connection with actual conditions of experience",
            Self::Necessity => "Determination by universal conditions of experience",
        }
    }

    /// Keywords that suggest this category is present in text.
    pub fn signal_words(&self) -> &'static [&'static str] {
        match self {
            Self::Unity => &[
                "one",
                "single",
                "unique",
                "alone",
                "only",
                "individual",
                "unified",
                "unit",
            ],
            Self::Plurality => &[
                "many",
                "multiple",
                "several",
                "various",
                "different",
                "diverse",
                "numerous",
            ],
            Self::Totality => &[
                "all",
                "every",
                "whole",
                "entire",
                "total",
                "complete",
                "universal",
                "each",
            ],
            Self::Reality => &[
                "is", "exists", "real", "actual", "present", "true", "positive", "has",
            ],
            Self::Negation => &[
                "not", "no", "never", "nothing", "none", "absent", "lack", "without", "deny",
                "false",
            ],
            Self::Limitation => &[
                "only", "merely", "limited", "partial", "some", "extent", "but", "however",
                "although",
            ],
            Self::Substance => &[
                "is",
                "substance",
                "thing",
                "object",
                "entity",
                "matter",
                "body",
                "remains",
                "persists",
                "underlying",
            ],
            Self::Causality => &[
                "because",
                "therefore",
                "causes",
                "leads",
                "results",
                "consequently",
                "thus",
                "hence",
                "since",
                "if",
                "then",
                "produces",
                "effect",
            ],
            Self::Community => &[
                "interact",
                "together",
                "mutual",
                "between",
                "reciprocal",
                "coexist",
                "relate",
                "simultaneously",
            ],
            Self::Possibility => &[
                "can",
                "could",
                "might",
                "possible",
                "may",
                "capable",
                "potential",
                "perhaps",
            ],
            Self::Existence => &[
                "exists", "is", "actual", "really", "indeed", "fact", "happens", "occurs",
            ],
            Self::Necessity => &[
                "must",
                "necessary",
                "inevitably",
                "always",
                "certainly",
                "cannot but",
                "required",
                "essential",
                "inevitably",
            ],
        }
    }

    /// Score how strongly this category is signalled in a proposition.
    pub fn score_in(&self, proposition: &Proposition) -> f64 {
        let text = proposition.text.to_lowercase();
        let words: Vec<&str> = text.split_whitespace().collect();
        let signals = self.signal_words();
        let hits = words.iter().filter(|&&w| signals.contains(&w)).count();
        // Normalize: log-scale to avoid dominance of long texts
        if hits == 0 {
            0.0
        } else {
            (hits as f64).ln() / (signals.len() as f64).ln() + (hits as f64 / words.len() as f64)
        }
    }
}

// ─── CategoryGroup ───────────────────────────────────────────────────────────

/// The four groups of categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CategoryGroup {
    Quantity,
    Quality,
    Relation,
    Modality,
}

impl CategoryGroup {
    pub fn categories(&self) -> [Category; 3] {
        match self {
            Self::Quantity => [Category::Unity, Category::Plurality, Category::Totality],
            Self::Quality => [Category::Reality, Category::Negation, Category::Limitation],
            Self::Relation => [
                Category::Substance,
                Category::Causality,
                Category::Community,
            ],
            Self::Modality => [
                Category::Possibility,
                Category::Existence,
                Category::Necessity,
            ],
        }
    }
}

// ─── CategoryApplication ─────────────────────────────────────────────────────

/// The result of applying a single category to a proposition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryApplication {
    pub category: Category,
    pub confidence: Confidence,
    /// Evidence: the signal words found in the text.
    pub evidence: Vec<String>,
}

// ─── CategoryAnalysis ────────────────────────────────────────────────────────

/// The result of applying all 12 categories to a set of propositions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryAnalysis {
    pub applications: Vec<CategoryApplication>,
    /// The dominant category (highest confidence).
    pub dominant: Option<Category>,
}

impl CategoryAnalysis {
    /// Analyze a set of propositions and score each category.
    pub fn from_propositions(propositions: &[Proposition]) -> Self {
        let mut applications: Vec<CategoryApplication> = Category::all()
            .iter()
            .map(|&cat| {
                let total_score: f64 = propositions.iter().map(|p| cat.score_in(p)).sum();
                let avg_score = if propositions.is_empty() {
                    0.0
                } else {
                    total_score / propositions.len() as f64
                };

                // Collect evidence words
                let evidence: Vec<String> = propositions
                    .iter()
                    .flat_map(|p| {
                        let text = p.text.to_lowercase();
                        cat.signal_words()
                            .iter()
                            .filter(|&&w| text.contains(w))
                            .map(|&w| w.to_string())
                            .collect::<Vec<_>>()
                    })
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();

                CategoryApplication {
                    category: cat,
                    confidence: Confidence::new(avg_score.min(1.0)),
                    evidence,
                }
            })
            .collect();

        // Sort by confidence descending
        applications.sort_by(|a, b| b.confidence.value().total_cmp(&a.confidence.value()));

        let dominant = applications
            .first()
            .filter(|a| a.confidence.value() > 0.0)
            .map(|a| a.category);

        Self {
            applications,
            dominant,
        }
    }

    /// Get applications above a confidence threshold.
    pub fn above_threshold(&self, threshold: f64) -> Vec<&CategoryApplication> {
        self.applications
            .iter()
            .filter(|a| a.confidence.value() >= threshold)
            .collect()
    }

    /// Analyze propositions with language-game context (TRIZ S-1).
    ///
    /// Multiplies base keyword scores by a game-specific weight matrix, producing
    /// differentiated category scores instead of the flat 0.06–0.10 range.
    /// All weights are derived from the semantic relationship between Kant's
    /// categories and Wittgenstein's forms of life — zero added dependencies.
    pub fn from_propositions_with_game(propositions: &[Proposition], game: FormOfLife) -> Self {
        let mut applications: Vec<CategoryApplication> = Category::all()
            .iter()
            .map(|&cat| {
                let total_score: f64 = propositions.iter().map(|p| cat.score_in(p)).sum();
                let base_score = if propositions.is_empty() {
                    0.0
                } else {
                    total_score / propositions.len() as f64
                };

                // Apply game-context weight (TRIZ S-1: Feedback + Local Quality)
                let weight = game_weight(game, cat);
                let weighted_score = (base_score * weight).min(1.0);

                let evidence: Vec<String> = propositions
                    .iter()
                    .flat_map(|p| {
                        let text = p.text.to_lowercase();
                        cat.signal_words()
                            .iter()
                            .filter(|&&w| text.contains(w))
                            .map(|&w| w.to_string())
                            .collect::<Vec<_>>()
                    })
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();

                CategoryApplication {
                    category: cat,
                    confidence: Confidence::new(weighted_score),
                    evidence,
                }
            })
            .collect();

        applications.sort_by(|a, b| b.confidence.value().total_cmp(&a.confidence.value()));

        let dominant = applications
            .first()
            .filter(|a| a.confidence.value() > 0.0)
            .map(|a| a.category);

        Self {
            applications,
            dominant,
        }
    }

    /// Compute a quick risk pre-score (0.0–1.0) from raw category confidence.
    ///
    /// Used by adaptive layer activation (TRIZ S-3) to decide whether the
    /// expensive Dialectic and Methodology layers should run.
    pub fn risk_pre_score(&self) -> f64 {
        // High-risk categories: Necessity (modal dogmatism), Totality (world-ideas),
        // Existence (theological overreach), Community (paralogism-prone).
        let risk_categories = [
            Category::Necessity,
            Category::Totality,
            Category::Existence,
            Category::Community,
        ];
        risk_categories
            .iter()
            .filter_map(|rc| self.applications.iter().find(|a| &a.category == rc))
            .map(|a| a.confidence.value())
            .fold(0.0_f64, f64::max)
    }
    /// Apply a Judgment×Category joint boost (TRIZ S-8).
    ///
    /// Certain judgment forms amplify the significance of certain categories.
    /// For example, a `Universal` + `Causality` co-occurrence signals a universal
    /// causal law (a scientific principle), which Kant places at the apex of
    /// theoretical knowledge — warranting a higher confidence score.
    ///
    /// Boosts are multiplicative (on top of game weights) and capped at 1.0.
    /// Only the dominant JudgmentForm produces a boost; tie cases produce no boost.
    pub fn apply_judgment_boost(&mut self, judgment: &JudgmentAnalysis) {
        let Some(dominant_form) = judgment.dominant else {
            return;
        };

        for app in self.applications.iter_mut() {
            let boost = judgment_category_boost(dominant_form, app.category);
            if boost > 1.0 {
                let new_val = (app.confidence.value() * boost).min(1.0);
                app.confidence = Confidence::new(new_val);
            }
        }

        // Re-sort and recompute dominant after boost
        self.applications
            .sort_by(|a, b| b.confidence.value().total_cmp(&a.confidence.value()));
        self.dominant = self
            .applications
            .first()
            .filter(|a| a.confidence.value() > 0.0)
            .map(|a| a.category);
    }
}

/// Judgment×Category joint boost table (TRIZ S-8).
///
/// Each cell maps a (JudgmentForm, Category) pair to a multiplier >1.0.
/// Pairs not listed return 1.0 (no boost). Derived from Kant's Table of Judgments
/// and the corresponding Table of Categories — the two tables are structurally
/// isomorphic; co-occurrence of a judgment form with its derived category is
/// philosophically significant.
///
/// Examples:
/// - Universal + Totality/Causality/Necessity → universal law or totality claim (high epistemic stakes)
/// - Apodeictic + Necessity → modal dogmatism (Kant's central concern)
/// - Categorical + Substance/Reality → is-predication asserting substance or reality
/// - Hypothetical + Causality/Existence → conditional causal reasoning
/// - Disjunctive + Plurality/Community → mutual exclusion, reciprocity
/// - Infinite + Limitation → infinite judgment expressing a limitative predicate
fn judgment_category_boost(form: JudgmentForm, category: Category) -> f64 {
    match (form, category) {
        // ── Universal judgment × categories (strongest epistemic claims)
        (JudgmentForm::Universal, Category::Totality) => 1.45,
        (JudgmentForm::Universal, Category::Causality) => 1.40,
        (JudgmentForm::Universal, Category::Necessity) => 1.35,
        (JudgmentForm::Universal, Category::Reality) => 1.25,

        // ── Apodeictic judgment (necessary/must) × modal and relational categories
        (JudgmentForm::Apodeictic, Category::Necessity) => 1.55,
        (JudgmentForm::Apodeictic, Category::Totality) => 1.40,
        (JudgmentForm::Apodeictic, Category::Causality) => 1.30,
        (JudgmentForm::Apodeictic, Category::Existence) => 1.25,

        // ── Categorical judgment (S is P) × substance and reality
        (JudgmentForm::Categorical, Category::Substance) => 1.35,
        (JudgmentForm::Categorical, Category::Reality) => 1.30,
        (JudgmentForm::Categorical, Category::Existence) => 1.25,

        // ── Hypothetical judgment (if P then Q) × causality and existence
        (JudgmentForm::Hypothetical, Category::Causality) => 1.30,
        (JudgmentForm::Hypothetical, Category::Existence) => 1.20,
        (JudgmentForm::Hypothetical, Category::Possibility) => 1.25,

        // ── Disjunctive judgment (either P or Q) × plurality and community
        (JudgmentForm::Disjunctive, Category::Plurality) => 1.25,
        (JudgmentForm::Disjunctive, Category::Community) => 1.30,
        (JudgmentForm::Disjunctive, Category::Limitation) => 1.20,

        // ── Singular judgment (this S is P) × existence and unity
        (JudgmentForm::Singular, Category::Existence) => 1.30,
        (JudgmentForm::Singular, Category::Unity) => 1.25,

        // ── Particular judgment (some S is P) × plurality and possibility
        (JudgmentForm::Particular, Category::Plurality) => 1.25,
        (JudgmentForm::Particular, Category::Possibility) => 1.20,

        // ── Negative judgment (S is not P) × negation
        (JudgmentForm::Negative, Category::Negation) => 1.40,
        (JudgmentForm::Negative, Category::Limitation) => 1.20,

        // ── Infinite judgment (S is non-P) × limitation
        (JudgmentForm::Infinite, Category::Limitation) => 1.45,
        (JudgmentForm::Infinite, Category::Negation) => 1.25,

        // ── Assertoric judgment (S actually is P) × existence and reality
        (JudgmentForm::Assertoric, Category::Existence) => 1.30,
        (JudgmentForm::Assertoric, Category::Reality) => 1.25,

        // ── Problematic judgment (S possibly is P) × possibility
        (JudgmentForm::Problematic, Category::Possibility) => 1.40,
        (JudgmentForm::Problematic, Category::Existence) => 1.15,

        _ => 1.0,
    }
}

/// Game-specific category weight matrix (TRIZ S-1: Principle #23 Feedback, #3 Local Quality).
///
/// Each cell gives the multiplier for (game, category). Values >1.0 boost a category
/// that is semantically primary in that language game. Value 1.0 = neutral.
///
/// Derived from philosophical analysis:
/// - Scientific game → Causality (hypothesis/effect), Existence (empirical), Substance (object)
/// - Moral game → Necessity (duty/must), Community (reciprocity), Totality (universal law)
/// - Mathematical game → Unity/Totality (axioms cover all), Necessity (proof), Limitation (bounds)
pub fn game_weight(game: FormOfLife, category: Category) -> f64 {
    match (game, category) {
        // Scientific: causal laws, existent objects, substances, necessary conditions
        (FormOfLife::Scientific, Category::Causality) => 1.85,
        (FormOfLife::Scientific, Category::Existence) => 1.65,
        (FormOfLife::Scientific, Category::Substance) => 1.55,
        (FormOfLife::Scientific, Category::Necessity) => 1.40,
        (FormOfLife::Scientific, Category::Reality) => 1.30,
        (FormOfLife::Scientific, Category::Totality) => 1.25,

        // Moral: duty (Necessity), reciprocity (Community), universal law (Totality)
        (FormOfLife::Moral, Category::Necessity) => 2.00,
        (FormOfLife::Moral, Category::Community) => 1.80,
        (FormOfLife::Moral, Category::Totality) => 1.60,
        (FormOfLife::Moral, Category::Reality) => 1.30,
        (FormOfLife::Moral, Category::Causality) => 1.20,

        // Mathematical: axioms (Unity), totality of proof (Totality), necessity of theorem
        (FormOfLife::Mathematical, Category::Unity) => 2.00,
        (FormOfLife::Mathematical, Category::Totality) => 1.90,
        (FormOfLife::Mathematical, Category::Necessity) => 1.90,
        (FormOfLife::Mathematical, Category::Limitation) => 1.75,
        (FormOfLife::Mathematical, Category::Plurality) => 1.50,

        // Religious: necessary being, existence of God, totality of creation
        (FormOfLife::Religious, Category::Necessity) => 1.85,
        (FormOfLife::Religious, Category::Existence) => 1.75,
        (FormOfLife::Religious, Category::Reality) => 1.60,
        (FormOfLife::Religious, Category::Totality) => 1.55,
        (FormOfLife::Religious, Category::Substance) => 1.40,

        // Technical: cause-effect chains, substance (objects), limitation (constraints)
        (FormOfLife::Technical, Category::Causality) => 1.75,
        (FormOfLife::Technical, Category::Substance) => 1.60,
        (FormOfLife::Technical, Category::Limitation) => 1.55,
        (FormOfLife::Technical, Category::Unity) => 1.35,
        (FormOfLife::Technical, Category::Existence) => 1.30,

        // Legal: necessity (obligation), community (mutual rights), totality (universal law)
        (FormOfLife::Legal, Category::Necessity) => 1.85,
        (FormOfLife::Legal, Category::Community) => 1.75,
        (FormOfLife::Legal, Category::Totality) => 1.65,
        (FormOfLife::Legal, Category::Causality) => 1.50,

        // Philosophical: substance and reality of concepts, necessity of argument
        (FormOfLife::Philosophical, Category::Reality) => 1.65,
        (FormOfLife::Philosophical, Category::Substance) => 1.60,
        (FormOfLife::Philosophical, Category::Necessity) => 1.55,
        (FormOfLife::Philosophical, Category::Existence) => 1.55,
        (FormOfLife::Philosophical, Category::Limitation) => 1.40,

        // Aesthetic/Narrative: unity of form, reality of expression, limitation (genre)
        (FormOfLife::Aesthetic, Category::Unity) => 1.55,
        (FormOfLife::Aesthetic, Category::Reality) => 1.45,
        (FormOfLife::Aesthetic, Category::Limitation) => 1.60,
        (FormOfLife::Aesthetic, Category::Plurality) => 1.30,

        (FormOfLife::Narrative, Category::Causality) => 1.65,
        (FormOfLife::Narrative, Category::Substance) => 1.50,
        (FormOfLife::Narrative, Category::Existence) => 1.45,
        (FormOfLife::Narrative, Category::Plurality) => 1.30,

        // Everyday/Unknown: neutral weights
        _ => 1.0,
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
    fn all_categories_are_twelve() {
        assert_eq!(Category::all().len(), 12);
    }

    #[test]
    fn causality_detected_in_causal_text() {
        let p = prop("Water boils because heat causes the temperature to rise");
        let analysis = CategoryAnalysis::from_propositions(&[p]);
        let causality = analysis
            .applications
            .iter()
            .find(|a| a.category == Category::Causality)
            .unwrap();
        assert!(
            causality.confidence.value() > 0.0,
            "Causality should be detected"
        );
    }

    #[test]
    fn negation_detected_in_negative_text() {
        let p = prop("There is nothing here and no evidence at all");
        let analysis = CategoryAnalysis::from_propositions(&[p]);
        let negation = analysis
            .applications
            .iter()
            .find(|a| a.category == Category::Negation)
            .unwrap();
        assert!(negation.confidence.value() > 0.0);
    }

    #[test]
    fn category_groups_are_correct() {
        assert_eq!(Category::Unity.group(), CategoryGroup::Quantity);
        assert_eq!(Category::Reality.group(), CategoryGroup::Quality);
        assert_eq!(Category::Causality.group(), CategoryGroup::Relation);
        assert_eq!(Category::Necessity.group(), CategoryGroup::Modality);
    }
}
