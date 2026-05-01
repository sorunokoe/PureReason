//! # Transcendental World Model (TRIZ S-1)
//!
//! A persistent cross-turn symbolic representation of the agent's "world" — a graph
//! of objects, their attributes (annotated by Kantian category), and causal rules.
//!
//! ## TRIZ Rationale
//! **S-1 (TC-2, TC-8, PC-4):**  
//! Resolves the contradiction between context-window coherence and cross-turn
//! contradiction density by extracting a compact *Unity Model* (not raw text) that
//! persists across turns. The Unity Model uses Kantian categories as the attribute
//! schema — Quantity (how many), Quality (what kind), Relation (causal links),
//! Modality (necessity/possibility).
//!
//! ## Kantian Grounding
//! The WorldModel is the computational implementation of Kant's
//! **synthetic unity of apperception** (CPR B131–136):
//! > "The I think must be capable of accompanying all my representations."
//! > All objects represented in discourse belong to a single coherent world-frame.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::analytic::Category;
use crate::pipeline::PipelineReport;

// ─── Object Identity ──────────────────────────────────────────────────────────

/// A stable identity for an entity in the world model.
/// Derived from a normalized form of the entity's name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub String);

impl ObjectId {
    /// Create an ObjectId by normalising a name: lowercase, trim, collapse spaces.
    pub fn from_name(name: &str) -> Self {
        ObjectId(
            name.trim()
                .to_lowercase()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" "),
        )
    }
}

impl std::fmt::Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ─── Fact ────────────────────────────────────────────────────────────────────

/// A time-stamped fact about an object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    /// Kantian category this fact falls under.
    pub category: Category,
    /// The value or description of the attribute.
    pub value: String,
    /// Conversation turn when this fact was recorded.
    pub time_step: usize,
    /// Whether this fact is asserted (+) or negated (−).
    pub polarity: bool,
}

// ─── WorldObject ──────────────────────────────────────────────────────────────

/// A persistent entity in the world model with categorised attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldObject {
    pub id: ObjectId,
    /// All facts ever asserted about this object (monotonically appended).
    pub facts: Vec<Fact>,
}

impl WorldObject {
    fn new(id: ObjectId) -> Self {
        WorldObject {
            id,
            facts: Vec::new(),
        }
    }

    /// Latest positive fact for a given category (most recent time-step).
    pub fn latest_fact(&self, cat: Category) -> Option<&Fact> {
        self.facts
            .iter()
            .rev()
            .find(|f| f.category == cat && f.polarity)
    }
}

// ─── CausalRule ───────────────────────────────────────────────────────────────

/// A causal regularity extracted from the discourse.
/// Interpreted as: "When *antecedent* holds, *consequent* tends to follow."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalRule {
    pub antecedent: String,
    pub consequent: String,
    /// Confidence in [0.0, 1.0].
    pub confidence: f64,
    /// Turn when this rule was first inferred.
    pub first_seen: usize,
}

// ─── UnityViolation ───────────────────────────────────────────────────────────

/// A detected violation of one of Kant's transcendental unity conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnityViolation {
    pub kind: UnityViolationKind,
    pub description: String,
    pub time_step: usize,
}

/// The type of unity condition that was violated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnityViolationKind {
    /// An object appeared, disappeared, and reappeared with contradictory attributes.
    ObjectPersistence,
    /// Two facts about the same object at the same time step contradict each other.
    ObjectCoherence,
    /// The temporal ordering of claimed events is inconsistent.
    TemporalCoherence,
    /// A causal loop was detected (A causes B causes ... causes A).
    CausalLoop,
    /// The same entity was treated as different ontological kinds across turns.
    CategoricalCoherence,
}

// ─── WorldModel ───────────────────────────────────────────────────────────────

/// The Transcendental World Model — the persistent cross-turn symbolic world.
///
/// Updated incrementally: only the delta (new propositions) is merged each turn.
/// Unity violations are accumulated; they do not remove prior facts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorldModel {
    /// All known objects, keyed by stable identity.
    pub objects: HashMap<ObjectId, WorldObject>,
    /// Causal rules extracted from the discourse.
    pub rules: Vec<CausalRule>,
    /// All detected unity violations.
    pub violations: Vec<UnityViolation>,
    /// Monotonic conversation clock (incremented each call to `update`).
    pub time_step: usize,
}

impl WorldModel {
    /// Create an empty world model.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the world model from a PipelineReport.
    ///
    /// This is the incremental merge step: extract objects and rules from the
    /// report, merge them into the existing world, and check unity conditions.
    pub fn update(&mut self, report: &PipelineReport) -> Vec<UnityViolation> {
        self.time_step += 1;
        let ts = self.time_step;

        // ── Extract propositions from the report ──────────────────────────────
        let dominant_cat = report
            .understanding
            .category_analysis
            .dominant
            .unwrap_or(Category::Existence);
        let input_lower = report.input.to_lowercase();

        // Detect polarity (rough heuristic)
        let negated = input_lower.contains(" not ")
            || input_lower.contains("no ")
            || input_lower.contains("never ")
            || input_lower.contains("isn't")
            || input_lower.contains("aren't")
            || input_lower.contains("doesn't")
            || input_lower.contains("cannot ")
            || input_lower.contains("can't ");

        // Extract noun-phrase subjects (first 1–3 capitalized words or first 3 words)
        let subjects = extract_subjects(&report.input);

        let mut new_violations = Vec::new();

        for subject in &subjects {
            let oid = ObjectId::from_name(subject);
            let entry = self
                .objects
                .entry(oid.clone())
                .or_insert_with(|| WorldObject::new(oid.clone()));

            // Check object coherence: does the new fact contradict an existing fact
            // at the SAME time-step (shouldn't happen, but guard for multi-sentence turns)?
            if let Some(existing) = entry.latest_fact(dominant_cat) {
                if existing.polarity == negated && existing.time_step == ts {
                    let v = UnityViolation {
                        kind: UnityViolationKind::ObjectCoherence,
                        description: format!(
                            "Object '{}' has contradictory '{}' facts at t={}",
                            oid,
                            category_name(dominant_cat),
                            ts
                        ),
                        time_step: ts,
                    };
                    new_violations.push(v.clone());
                    self.violations.push(v);
                }
            }

            // Record the new fact
            entry.facts.push(Fact {
                category: dominant_cat,
                value: report.input.clone(),
                time_step: ts,
                polarity: !negated,
            });
        }

        // ── Extract causal rules ──────────────────────────────────────────────
        if dominant_cat == Category::Causality
            || input_lower.contains(" causes ")
            || input_lower.contains(" leads to ")
            || input_lower.contains(" results in ")
        {
            if let Some(rule) = extract_causal_rule(&report.input, ts) {
                // Check for causal loops before inserting
                if self.would_create_causal_loop(&rule) {
                    let v = UnityViolation {
                        kind: UnityViolationKind::CausalLoop,
                        description: format!(
                            "Causal loop detected: '{}' → '{}' forms a cycle",
                            rule.antecedent, rule.consequent
                        ),
                        time_step: ts,
                    };
                    new_violations.push(v.clone());
                    self.violations.push(v);
                } else {
                    self.rules.push(rule);
                }
            }
        }

        new_violations
    }

    /// Check whether adding this causal rule would create a cycle.
    fn would_create_causal_loop(&self, candidate: &CausalRule) -> bool {
        // DFS: starting from consequent, can we reach the antecedent?
        let target = candidate.antecedent.to_lowercase();
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![candidate.consequent.to_lowercase()];

        while let Some(node) = stack.pop() {
            if node == target {
                return true;
            }
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node.clone());
            for rule in &self.rules {
                if rule.antecedent.to_lowercase() == node {
                    stack.push(rule.consequent.to_lowercase());
                }
            }
        }
        false
    }

    /// Predict propositions that SHOULD follow from the current WorldModel.
    /// Used by the Synthetic A Priori Validator.
    pub fn predict_next(&self) -> Vec<String> {
        let mut predictions = Vec::new();
        let ts = self.time_step;

        // For every object with a Substance fact, predict persistence
        for obj in self.objects.values() {
            if let Some(f) = obj.latest_fact(Category::Substance) {
                if f.time_step == ts {
                    predictions.push(format!("{} continues to exist", obj.id));
                }
            }
        }

        // For every causal rule with a recently asserted antecedent, predict the consequent
        for rule in &self.rules {
            let ant = rule.antecedent.to_lowercase();
            let recently_asserted = self.objects.values().any(|o| {
                o.facts
                    .iter()
                    .rev()
                    .take(3)
                    .any(|f| f.value.to_lowercase().contains(&ant) && f.polarity)
            });
            if recently_asserted && rule.confidence >= 0.5 {
                predictions.push(rule.consequent.clone());
            }
        }

        predictions
    }

    /// Retrodict: what facts held at a given time step?
    pub fn retrodict(&self, at_time: usize) -> Vec<(ObjectId, Fact)> {
        let mut result = Vec::new();
        for obj in self.objects.values() {
            for fact in &obj.facts {
                if fact.time_step == at_time {
                    result.push((obj.id.clone(), fact.clone()));
                }
            }
        }
        result
    }

    /// Return a human-readable summary of the world model.
    pub fn summary(&self) -> String {
        let obj_count = self.objects.len();
        let rule_count = self.rules.len();
        let violation_count = self.violations.len();
        format!(
            "WorldModel[t={}]: {} object(s), {} causal rule(s), {} unity violation(s)",
            self.time_step, obj_count, rule_count, violation_count
        )
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Extract likely subject noun phrases from text (heuristic).
fn extract_subjects(text: &str) -> Vec<String> {
    let mut subjects = Vec::new();

    // Collect capitalised or first-word candidates
    for sentence in text.split(['.', '!', '?']) {
        let s = sentence.trim();
        if s.is_empty() {
            continue;
        }

        // First 1–3 words of sentence as subject candidate
        let words: Vec<&str> = s.split_whitespace().collect();
        if words.is_empty() {
            continue;
        }

        // If first word is a known pronoun, skip (generic subject)
        let first = words[0].to_lowercase();
        if matches!(
            first.as_str(),
            "i" | "we" | "they" | "it" | "this" | "that" | "there"
        ) {
            // Use a generic "self" object for first-person
            if first == "i" {
                subjects.push("self".to_string());
            }
            continue;
        }

        // Take up to 3 words as noun phrase
        let np: String = words
            .iter()
            .take(3)
            .map(|w| w.trim_matches(|c: char| !c.is_alphabetic()))
            .filter(|w| !w.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        if !np.is_empty() {
            subjects.push(np);
        }
    }

    if subjects.is_empty() {
        // Fallback: use full text truncated to 30 chars as subject key
        subjects.push(text.chars().take(30).collect());
    }

    subjects
}

/// Extract a causal rule from text if possible.
fn extract_causal_rule(text: &str, ts: usize) -> Option<CausalRule> {
    let patterns = [
        (" causes ", 0.9),
        (" leads to ", 0.85),
        (" results in ", 0.85),
        (" produces ", 0.8),
        (" implies ", 0.75),
        (" because ", 0.7),
        (" therefore ", 0.7),
        (" thus ", 0.65),
    ];

    for (pat, confidence) in &patterns {
        if let Some(pos) = text.to_lowercase().find(pat) {
            let antecedent = text[..pos].trim().to_string();
            let consequent = text[pos + pat.len()..]
                .trim()
                .trim_end_matches(['.', '!', '?'])
                .to_string();
            if !antecedent.is_empty() && !consequent.is_empty() {
                return Some(CausalRule {
                    antecedent,
                    consequent,
                    confidence: *confidence,
                    first_seen: ts,
                });
            }
        }
    }
    None
}

fn category_name(c: Category) -> &'static str {
    match c {
        Category::Unity => "Unity",
        Category::Plurality => "Plurality",
        Category::Totality => "Totality",
        Category::Reality => "Reality",
        Category::Negation => "Negation",
        Category::Limitation => "Limitation",
        Category::Substance => "Substance",
        Category::Causality => "Causality",
        Category::Community => "Community",
        Category::Possibility => "Possibility",
        Category::Existence => "Existence",
        Category::Necessity => "Necessity",
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_report(input: &str, cat: Category) -> PipelineReport {
        use crate::pipeline::KantianPipeline;
        let p = KantianPipeline::new();
        let mut r = p.process(input).unwrap();
        // Force the dominant category for deterministic tests
        r.understanding.category_analysis.dominant = Some(cat);
        r
    }

    #[test]
    fn world_model_update_adds_object() {
        let mut wm = WorldModel::new();
        let r = make_report("The bridge is strong.", Category::Substance);
        wm.update(&r);
        assert_eq!(wm.time_step, 1);
        assert!(!wm.objects.is_empty());
    }

    #[test]
    fn world_model_extracts_causal_rule() {
        let mut wm = WorldModel::new();
        let r = make_report("Rain causes flooding.", Category::Causality);
        wm.update(&r);
        assert!(!wm.rules.is_empty());
        let rule = &wm.rules[0];
        assert!(rule.antecedent.to_lowercase().contains("rain"));
        assert!(rule.consequent.to_lowercase().contains("flood"));
    }

    #[test]
    fn world_model_detects_causal_loop() {
        let mut wm = WorldModel::new();
        // A → B
        let r1 = make_report("A causes B.", Category::Causality);
        wm.update(&r1);
        // B → A (loop!)
        let r2 = make_report("B causes A.", Category::Causality);
        let violations = wm.update(&r2);
        assert!(violations
            .iter()
            .any(|v| v.kind == UnityViolationKind::CausalLoop));
    }

    #[test]
    fn world_model_predict_next_returns_predictions() {
        let mut wm = WorldModel::new();
        let r = make_report("Heat causes expansion.", Category::Causality);
        wm.update(&r);
        let preds = wm.predict_next();
        // Should predict the consequent given the antecedent
        assert!(preds.iter().any(|p| p.to_lowercase().contains("expansion")));
    }

    #[test]
    fn world_model_retrodict_returns_facts_at_time() {
        let mut wm = WorldModel::new();
        let r1 = make_report("Water is liquid.", Category::Substance);
        wm.update(&r1); // t=1
        let r2 = make_report("Ice is solid.", Category::Substance);
        wm.update(&r2); // t=2
        let at_t1 = wm.retrodict(1);
        assert!(!at_t1.is_empty());
        let all_t1_values: Vec<_> = at_t1.iter().map(|(_, f)| f.value.to_lowercase()).collect();
        assert!(all_t1_values.iter().any(|v| v.contains("water")));
    }
}
