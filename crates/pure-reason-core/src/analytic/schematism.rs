//! # Schematism of the Pure Understanding
//!
//! The Schematism is one of Kant's most important and difficult doctrines.
//! It solves the problem of how pure categories (which are purely intellectual)
//! can apply to empirical intuitions (which are sensible).
//!
//! The bridge is the **Schema**: a transcendental determination of time that
//! is both intellectual (like a category) and sensible (like an intuition).
//!
//! "The schema is in itself always a product of the imagination." — CPR A140/B179
//!
//! ## The Schemas (one per category):
//!
//! | Category | Schema (Temporal Determination) |
//! |----------|--------------------------------|
//! | Unity | Number (iteration in time) |
//! | Plurality | Number |
//! | Totality | Number (completeness) |
//! | Reality | Degree (filling of time) |
//! | Negation | Empty time / zero degree |
//! | Limitation | Bounded degree |
//! | Substance | Permanence in time |
//! | Causality | Temporal succession (rule-governed) |
//! | Community | Simultaneity / coexistence |
//! | Possibility | Agreement with conditions of time |
//! | Existence | Existence at some time |
//! | Necessity | Existence at all times |

use super::categories::{Category, CategoryAnalysis};
use crate::aesthetic::time::TimeForm;
use serde::{Deserialize, Serialize};

// ─── TemporalDetermination ───────────────────────────────────────────────────

/// The temporal determination that constitutes the Schema for each category.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporalDetermination {
    /// Number — a rule for generating a series of homogeneous units (Quantity).
    Number,
    /// Degree — the filling of time with sensation intensity (Quality).
    Degree,
    /// Permanence — that which exists throughout all change (Substance).
    Permanence,
    /// Succession — determinate time-order, A before B (Causality).
    Succession,
    /// Simultaneity — co-existence in the same time (Community).
    Simultaneity,
    /// Sometime — agreement with existence at some time point (Possibility).
    Sometime,
    /// Sometime-actual — existence at a definite time (Existence).
    SometimeActual,
    /// All-time — existence at all times without exception (Necessity).
    AllTime,
}

impl TemporalDetermination {
    /// A human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Number => "Iteration or counting in time — the schema of quantity",
            Self::Degree => "The intensive filling of a time-moment — the schema of quality",
            Self::Permanence => "That which exists throughout all time — the schema of substance",
            Self::Succession => "Determinate time-order, rule-governed — the schema of causality",
            Self::Simultaneity => "Simultaneous co-existence in time — the schema of community",
            Self::Sometime => "Existence at some possible time — the schema of possibility",
            Self::SometimeActual => {
                "Existence at an actual, definite time — the schema of existence"
            }
            Self::AllTime => "Existence at all times without exception — the schema of necessity",
        }
    }
}

// ─── Schema ──────────────────────────────────────────────────────────────────

/// A Schema is the temporal determination of a pure category.
///
/// It is the "third thing" between pure concept and empirical intuition —
/// a transcendental time-determination that makes their union possible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// The category being schematized.
    pub category: Category,
    /// The temporal determination that constitutes this schema.
    pub determination: TemporalDetermination,
    /// How this schema is instantiated in the given temporal context.
    pub instantiation: SchemaInstantiation,
}

/// How a schema is concretely instantiated in a specific context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInstantiation {
    /// Whether the temporal condition is met in the text.
    pub condition_met: bool,
    /// Evidence from the TimeForm supporting or denying this schema.
    pub temporal_evidence: Vec<String>,
    /// A human-readable description of how the schema applies.
    pub description: String,
}

// ─── Schematism ──────────────────────────────────────────────────────────────

/// The faculty of Schematism — bridges categories and intuitions via time.
pub struct Schematism;

impl Schematism {
    pub fn new() -> Self {
        Self
    }

    /// The temporal determination (schema) for each category.
    pub fn schema_for(category: Category) -> TemporalDetermination {
        match category {
            Category::Unity | Category::Plurality | Category::Totality => {
                TemporalDetermination::Number
            }
            Category::Reality => TemporalDetermination::Degree,
            Category::Negation => TemporalDetermination::Degree, // zero degree
            Category::Limitation => TemporalDetermination::Degree, // bounded degree
            Category::Substance => TemporalDetermination::Permanence,
            Category::Causality => TemporalDetermination::Succession,
            Category::Community => TemporalDetermination::Simultaneity,
            Category::Possibility => TemporalDetermination::Sometime,
            Category::Existence => TemporalDetermination::SometimeActual,
            Category::Necessity => TemporalDetermination::AllTime,
        }
    }

    /// Apply schematism to a CategoryAnalysis using the TimeForm context.
    pub fn apply_to_analysis(&self, analysis: &CategoryAnalysis, time: &TimeForm) -> Vec<Schema> {
        analysis
            .applications
            .iter()
            .filter(|app| app.confidence.value() > 0.0)
            .map(|app| {
                let determination = Self::schema_for(app.category);
                let instantiation = self.instantiate(&determination, time, &app.evidence);
                Schema {
                    category: app.category,
                    determination,
                    instantiation,
                }
            })
            .collect()
    }

    /// Instantiate a temporal determination against a TimeForm.
    fn instantiate(
        &self,
        determination: &TemporalDetermination,
        time: &TimeForm,
        evidence: &[String],
    ) -> SchemaInstantiation {
        use crate::aesthetic::time::TemporalMarkerKind;

        let (condition_met, temporal_evidence, description) = match determination {
            TemporalDetermination::Number => {
                let met = !time.events.is_empty();
                let ev: Vec<String> = time
                    .events
                    .iter()
                    .take(3)
                    .map(|e| e.description.clone())
                    .collect();
                (
                    met,
                    ev,
                    format!(
                        "Quantity schema: {} discrete temporal events found",
                        time.events.len()
                    ),
                )
            }
            TemporalDetermination::Succession => {
                let has_sequence = time
                    .markers
                    .iter()
                    .any(|m| m.kind == TemporalMarkerKind::Sequential);
                let ev: Vec<String> = time
                    .markers
                    .iter()
                    .filter(|m| m.kind == TemporalMarkerKind::Sequential)
                    .map(|m| m.text.clone())
                    .collect();
                (
                    has_sequence,
                    ev,
                    "Causality schema: sequential temporal markers found".to_string(),
                )
            }
            TemporalDetermination::Permanence => {
                let has_present = time
                    .markers
                    .iter()
                    .any(|m| m.kind == TemporalMarkerKind::Present);
                let ev: Vec<String> = time
                    .markers
                    .iter()
                    .filter(|m| m.kind == TemporalMarkerKind::Present)
                    .map(|m| m.text.clone())
                    .collect();
                (
                    has_present,
                    ev,
                    "Substance schema: permanence-indicating markers".to_string(),
                )
            }
            TemporalDetermination::Simultaneity => {
                let has_coexistence = time.orderings.is_empty() && !time.events.is_empty();
                (
                    has_coexistence,
                    Vec::new(),
                    "Community schema: events appear coexistent".to_string(),
                )
            }
            TemporalDetermination::Degree => {
                // Reality/Negation/Limitation — check for intensity markers in evidence
                let intensity_words = [
                    "very",
                    "quite",
                    "extremely",
                    "barely",
                    "somewhat",
                    "partially",
                ];
                let ev: Vec<String> = evidence
                    .iter()
                    .filter(|e| intensity_words.iter().any(|w| e.contains(w)))
                    .cloned()
                    .collect();
                (
                    true,
                    ev,
                    "Quality schema: degree/intensity determination".to_string(),
                )
            }
            TemporalDetermination::Sometime => {
                let has_possible = time.markers.iter().any(|m| {
                    m.kind == TemporalMarkerKind::Future || m.kind == TemporalMarkerKind::Past
                });
                (
                    has_possible,
                    Vec::new(),
                    "Possibility schema: existence at some time".to_string(),
                )
            }
            TemporalDetermination::SometimeActual => {
                let has_present = time
                    .markers
                    .iter()
                    .any(|m| m.kind == TemporalMarkerKind::Present);
                (
                    has_present,
                    Vec::new(),
                    "Existence schema: existence at actual present time".to_string(),
                )
            }
            TemporalDetermination::AllTime => {
                let has_universal_time = evidence
                    .iter()
                    .any(|e| matches!(e.as_str(), "always" | "never" | "eternally" | "invariably"));
                (
                    has_universal_time,
                    Vec::new(),
                    "Necessity schema: existence at all times".to_string(),
                )
            }
        };

        SchemaInstantiation {
            condition_met,
            temporal_evidence,
            description,
        }
    }
}

impl Default for Schematism {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_for_causality_is_succession() {
        assert_eq!(
            Schematism::schema_for(Category::Causality),
            TemporalDetermination::Succession
        );
    }

    #[test]
    fn schema_for_substance_is_permanence() {
        assert_eq!(
            Schematism::schema_for(Category::Substance),
            TemporalDetermination::Permanence
        );
    }

    #[test]
    fn schema_for_necessity_is_all_time() {
        assert_eq!(
            Schematism::schema_for(Category::Necessity),
            TemporalDetermination::AllTime
        );
    }

    #[test]
    fn all_categories_have_schemas() {
        for cat in Category::all() {
            // Just ensure no panic
            let _ = Schematism::schema_for(cat);
        }
    }
}
