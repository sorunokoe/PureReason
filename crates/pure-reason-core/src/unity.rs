//! # Transcendental Unity Checker (TRIZ S-5 upgrade)
//!
//! Upgrades the simple Vepol-triple cross-turn contradiction detection to a full
//! Kantian unity check, implementing the **four unity conditions**:
//!
//! 1. **Object Coherence** — same object cannot have contradictory attributes in
//!    the same turn
//! 2. **Temporal Coherence** — stated event times must be monotonically consistent
//! 3. **Causal Coherence** — no causal loops (A → B → … → A)
//! 4. **Categorical Coherence** — the same entity cannot shift ontological kind
//!    across turns (event ↔ substance)
//!
//! ## TRIZ Rationale
//! **S-5 (TC-2, PC-1):**  
//! The EpistemicState (TRIZ S-2) detected polarity contradictions via Vepol triples.
//! This module upgrades to the *full* synthetic unity of apperception:
//! > "Kant's Copernican revolution: data conforms to cognitive structure."
//! > Violations are returned so the pipeline can inject regulative corrections.

use crate::analytic::Category;
use crate::pipeline::PipelineReport;
use crate::world_model::{UnityViolation, UnityViolationKind, WorldModel};

// ─── UnityCondition trait ────────────────────────────────────────────────────

/// A single Kantian unity condition applied to the world model + new report.
pub trait UnityCondition: Send + Sync {
    /// Check the condition; return any violations found.
    fn check(&self, world: &WorldModel, new_report: &PipelineReport) -> Vec<UnityViolation>;
    /// Short name for diagnostics.
    fn name(&self) -> &'static str;
}

// ─── Condition 1: Object Coherence ───────────────────────────────────────────

/// Objects cannot have directly contradictory facts at the same time step.
pub struct ObjectCoherenceCondition;

impl UnityCondition for ObjectCoherenceCondition {
    fn name(&self) -> &'static str {
        "ObjectCoherence"
    }

    fn check(&self, world: &WorldModel, new_report: &PipelineReport) -> Vec<UnityViolation> {
        let mut violations = Vec::new();
        let dominant_cat = match new_report.understanding.category_analysis.dominant {
            Some(c) => c,
            None => return violations,
        };
        let text_lower = new_report.input.to_lowercase();
        let negated = is_negated(&text_lower);

        // Check whether any existing object has a contradicting recent fact
        for obj in world.objects.values() {
            // The object is "mentioned" if its id appears in the new text
            if !text_lower.contains(&obj.id.0) {
                continue;
            }

            if let Some(f) = obj.latest_fact(dominant_cat) {
                if f.polarity == negated {
                    violations.push(UnityViolation {
                        kind: UnityViolationKind::ObjectCoherence,
                        description: format!(
                            "Object '{}' is now asserted {} but was previously asserted {}  \
                             under category '{:?}'",
                            obj.id,
                            if negated { "false" } else { "true" },
                            if f.polarity { "true" } else { "false" },
                            dominant_cat
                        ),
                        time_step: world.time_step + 1,
                    });
                }
            }
        }
        violations
    }
}

// ─── Condition 2: Temporal Coherence ─────────────────────────────────────────

/// Event sequences must be temporally consistent (no anachronisms).
pub struct TemporalCoherenceCondition;

impl UnityCondition for TemporalCoherenceCondition {
    fn name(&self) -> &'static str {
        "TemporalCoherence"
    }

    fn check(&self, world: &WorldModel, new_report: &PipelineReport) -> Vec<UnityViolation> {
        let mut violations = Vec::new();
        let text = new_report.input.to_lowercase();

        // Look for explicit past-tense claims about things stated in the future earlier
        let past_signals = [
            "used to",
            "was ",
            "were ",
            "had ",
            "previously ",
            "before ",
            "ago",
        ];
        let future_signals = ["will be", "is going to", "shall be", "future", "upcoming"];

        let claims_past = past_signals.iter().any(|s| text.contains(s));
        let claims_future = future_signals.iter().any(|s| text.contains(s));

        if claims_past && claims_future {
            violations.push(UnityViolation {
                kind: UnityViolationKind::TemporalCoherence,
                description: format!(
                    "Turn {} simultaneously references past and future tense for the same subject — \
                     temporal coherence may be violated",
                    world.time_step + 1
                ),
                time_step: world.time_step + 1,
            });
        }

        // Check if something previously claimed to be necessary is now denied
        for obj in world.objects.values() {
            if !text.contains(&obj.id.0) {
                continue;
            }
            if let Some(f) = obj.latest_fact(Category::Necessity) {
                if f.polarity {
                    let negated = is_negated(&text);
                    if negated {
                        violations.push(UnityViolation {
                            kind: UnityViolationKind::TemporalCoherence,
                            description: format!(
                                "Object '{}' was previously asserted as necessary but is now denied",
                                obj.id
                            ),
                            time_step: world.time_step + 1,
                        });
                    }
                }
            }
        }
        violations
    }
}

// ─── Condition 3: Causal Coherence ───────────────────────────────────────────

/// Causal claims must not introduce cycles.
pub struct CausalCoherenceCondition;

impl UnityCondition for CausalCoherenceCondition {
    fn name(&self) -> &'static str {
        "CausalCoherence"
    }

    fn check(&self, world: &WorldModel, _new_report: &PipelineReport) -> Vec<UnityViolation> {
        // The WorldModel.update() already performs cycle detection.
        // This condition re-checks the existing violations list for causal loops
        // and surfaces any from the current turn.
        world
            .violations
            .iter()
            .filter(|v| v.kind == UnityViolationKind::CausalLoop && v.time_step == world.time_step)
            .cloned()
            .collect()
    }
}

// ─── Condition 4: Categorical Coherence ──────────────────────────────────────

/// An entity should not shift ontological kind across turns
/// (e.g., event ↔ substance ↔ property).
pub struct CategoricalCoherenceCondition;

impl UnityCondition for CategoricalCoherenceCondition {
    fn name(&self) -> &'static str {
        "CategoricalCoherence"
    }

    fn check(&self, world: &WorldModel, new_report: &PipelineReport) -> Vec<UnityViolation> {
        let mut violations = Vec::new();
        let text_lower = new_report.input.to_lowercase();
        let new_cat = match new_report.understanding.category_analysis.dominant {
            Some(c) => c,
            None => return violations,
        };

        for obj in world.objects.values() {
            if !text_lower.contains(&obj.id.0) {
                continue;
            }

            // Find the most common category for this object historically.
            //
            // BTreeMap + explicit tie-break on key keeps the result
            // deterministic across runs (HashMap iteration order is randomized
            // in Rust, which made this function non-deterministic on ties — see
            // TRIZ-42 NE-9).
            let mut cat_counts: std::collections::BTreeMap<u8, usize> =
                std::collections::BTreeMap::new();
            for f in &obj.facts {
                *cat_counts.entry(f.category as u8).or_insert(0) += 1;
            }
            if let Some((&dominant_byte, _)) = cat_counts
                .iter()
                .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
            {
                let historical_kind = ontological_kind(category_from_u8(dominant_byte));
                let new_kind = ontological_kind(new_cat);
                if historical_kind != new_kind {
                    violations.push(UnityViolation {
                        kind: UnityViolationKind::CategoricalCoherence,
                        description: format!(
                            "Object '{}' shifts ontological kind from {:?} to {:?}",
                            obj.id, historical_kind, new_kind
                        ),
                        time_step: world.time_step + 1,
                    });
                }
            }
        }
        violations
    }
}

// ─── UnityChecker ─────────────────────────────────────────────────────────────

/// Runs all four unity conditions against the world model + new report.
pub struct UnityChecker {
    conditions: Vec<Box<dyn UnityCondition>>,
}

impl UnityChecker {
    /// Create a checker with the standard four Kantian unity conditions.
    pub fn new() -> Self {
        UnityChecker {
            conditions: vec![
                Box::new(ObjectCoherenceCondition),
                Box::new(TemporalCoherenceCondition),
                Box::new(CausalCoherenceCondition),
                Box::new(CategoricalCoherenceCondition),
            ],
        }
    }

    /// Run all conditions and return any violations found.
    pub fn check_all(&self, world: &WorldModel, report: &PipelineReport) -> Vec<UnityViolation> {
        self.conditions
            .iter()
            .flat_map(|c| c.check(world, report))
            .collect()
    }

    /// Check and update the world model in one step.
    /// Returns all violations (from conditions + world model update).
    pub fn update_and_check(
        &self,
        world: &mut WorldModel,
        report: &PipelineReport,
    ) -> Vec<UnityViolation> {
        // Run pre-update checks (before the new facts are merged)
        let pre_violations = self.check_all(world, report);
        // Update world model (runs its own internal checks)
        let update_violations = world.update(report);
        // Deduplicate (by description)
        let mut all = pre_violations;
        for v in update_violations {
            if !all.iter().any(|x| x.description == v.description) {
                all.push(v);
            }
        }
        all
    }
}

impl Default for UnityChecker {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn is_negated(text: &str) -> bool {
    text.contains(" not ")
        || text.contains("no ")
        || text.contains("never ")
        || text.contains("isn't")
        || text.contains("aren't")
        || text.contains("doesn't")
        || text.contains("cannot ")
        || text.contains("can't ")
}

/// Coarse ontological kind — used to detect categorical shifts.
#[derive(Debug, Clone, Copy, PartialEq)]
enum OntologicalKind {
    Entity,
    Event,
    Property,
    Modal,
}

fn ontological_kind(cat: Category) -> OntologicalKind {
    match cat {
        Category::Substance | Category::Unity | Category::Plurality | Category::Totality => {
            OntologicalKind::Entity
        }
        Category::Causality | Category::Community => OntologicalKind::Event,
        Category::Reality | Category::Negation | Category::Limitation => OntologicalKind::Property,
        Category::Possibility | Category::Existence | Category::Necessity => OntologicalKind::Modal,
    }
}

fn category_from_u8(v: u8) -> Category {
    match v {
        0 => Category::Unity,
        1 => Category::Plurality,
        2 => Category::Totality,
        3 => Category::Reality,
        4 => Category::Negation,
        5 => Category::Limitation,
        6 => Category::Substance,
        7 => Category::Causality,
        8 => Category::Community,
        9 => Category::Possibility,
        10 => Category::Existence,
        _ => Category::Necessity,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::KantianPipeline;
    use crate::world_model::WorldModel;

    fn make_report(input: &str, cat: Category) -> PipelineReport {
        let p = KantianPipeline::new();
        let mut r = p.process(input).unwrap();
        r.understanding.category_analysis.dominant = Some(cat);
        r
    }

    #[test]
    fn unity_checker_no_violations_on_fresh_world() {
        let world = WorldModel::new();
        let checker = UnityChecker::new();
        let r = make_report("The bridge is strong.", Category::Substance);
        let violations = checker.check_all(&world, &r);
        assert!(violations.is_empty());
    }

    #[test]
    fn unity_checker_detects_object_coherence_violation() {
        let mut world = WorldModel::new();
        let checker = UnityChecker::new();

        let r1 = make_report("The bridge is strong.", Category::Substance);
        checker.update_and_check(&mut world, &r1);

        // Now contradict: bridge is NOT strong
        let r2 = make_report("The bridge is not strong.", Category::Substance);
        let violations = checker.update_and_check(&mut world, &r2);
        // The object-coherence check should catch this
        assert!(violations
            .iter()
            .any(|v| v.kind == UnityViolationKind::ObjectCoherence));
    }

    #[test]
    fn unity_checker_detects_temporal_inconsistency() {
        let world = WorldModel::new();
        let checker = UnityChecker::new();
        let r = make_report(
            "The event was completed before it will start in the future.",
            Category::Causality,
        );
        let violations = checker.check_all(&world, &r);
        assert!(violations
            .iter()
            .any(|v| v.kind == UnityViolationKind::TemporalCoherence));
    }

    #[test]
    fn unity_checker_no_false_positive_on_consistent_facts() {
        let mut world = WorldModel::new();
        let checker = UnityChecker::new();
        let r1 = make_report("Water is liquid.", Category::Substance);
        checker.update_and_check(&mut world, &r1);
        let r2 = make_report("Ice is solid.", Category::Substance);
        let violations = checker.update_and_check(&mut world, &r2);
        // Different objects — should not produce coherence violations
        assert!(!violations
            .iter()
            .any(|v| v.kind == UnityViolationKind::ObjectCoherence));
    }
}
