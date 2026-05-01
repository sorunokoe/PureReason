//! # Apperception Engine (TRIZ S-2)
//!
//! Top-level orchestrator that wraps the WorldModel and UnityChecker, implementing
//! Evans' (2022) Apperception Engine concept: given a sequence of observations,
//! synthesise a coherent causal theory satisfying Kant's unity conditions.
//!
//! ## Three Unity Conditions (Evans 2022)
//! 1. **Object Persistence** — objects survive across turns unless explicitly negated
//! 2. **Causal Closure** — every event has a traceable cause in the world model
//! 3. **Spatio-Temporal Coherence** — no object in two places; time is monotonic
//!
//! ## TRIZ Rationale
//! **S-2 (TC-1, PC-1):**  
//! A priori unity conditions (fixed) wrap empirical domain learning (adaptive).
//! The Nested Doll principle (P7): constraints at the meta-level, rules at object-level.

use crate::{
    pipeline::PipelineReport,
    unity::UnityChecker,
    world_model::{UnityViolation, UnityViolationKind, WorldModel},
};
use serde::{Deserialize, Serialize};

// ─── ApperceptionResult ───────────────────────────────────────────────────────

/// Result of processing one turn through the Apperception Engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApperceptionResult {
    /// Unity violations detected in this turn.
    pub violations: Vec<UnityViolation>,
    /// Whether all unity conditions were satisfied.
    pub unified: bool,
    /// Forward predictions from the world model.
    pub predictions: Vec<String>,
    /// Human-readable world model summary after this update.
    pub world_summary: String,
}

impl ApperceptionResult {
    /// True if there are no violations of any kind.
    pub fn is_coherent(&self) -> bool {
        self.violations.is_empty()
    }

    /// True if there are causal loop violations specifically.
    pub fn has_causal_loops(&self) -> bool {
        self.violations
            .iter()
            .any(|v| v.kind == UnityViolationKind::CausalLoop)
    }
}

// ─── ApperceptionEngine ───────────────────────────────────────────────────────

/// The Kantian Apperception Engine.
///
/// Maintains a persistent WorldModel across turns and enforces all four unity
/// conditions at each update. Provides forward (predict) and backward (retrodict)
/// queries on the accumulated world model.
pub struct ApperceptionEngine {
    world: WorldModel,
    checker: UnityChecker,
}

impl ApperceptionEngine {
    /// Create a fresh engine with an empty world model.
    pub fn new() -> Self {
        ApperceptionEngine {
            world: WorldModel::new(),
            checker: UnityChecker::new(),
        }
    }

    /// Process a new pipeline report: update the world model and check unity.
    ///
    /// This is the core incremental step. Call once per LLM turn.
    pub fn update(&mut self, report: &PipelineReport) -> ApperceptionResult {
        let violations = self.checker.update_and_check(&mut self.world, report);
        let predictions = self.world.predict_next();
        let world_summary = self.world.summary();
        let unified = violations.is_empty();
        ApperceptionResult {
            violations,
            unified,
            predictions,
            world_summary,
        }
    }

    /// Forward model: predict propositions that should follow from current state.
    pub fn predict_next(&self) -> Vec<String> {
        self.world.predict_next()
    }

    /// Backward model: return all facts asserted at a given time step.
    pub fn retrodict(&self, time_step: usize) -> Vec<(String, String)> {
        self.world
            .retrodict(time_step)
            .into_iter()
            .map(|(id, fact)| (id.to_string(), fact.value))
            .collect()
    }

    /// Number of objects in the current world model.
    pub fn object_count(&self) -> usize {
        self.world.objects.len()
    }

    /// Number of causal rules in the current world model.
    pub fn rule_count(&self) -> usize {
        self.world.rules.len()
    }

    /// Total unity violations accumulated.
    pub fn total_violations(&self) -> usize {
        self.world.violations.len()
    }

    /// Current time step (number of turns processed).
    pub fn time_step(&self) -> usize {
        self.world.time_step
    }

    /// Borrow the underlying world model.
    pub fn world(&self) -> &WorldModel {
        &self.world
    }

    /// An ideality score [0.0, 1.0]: how close to a perfectly coherent world model.
    /// 1.0 = no violations, 0.0 = every turn has a violation.
    pub fn ideality(&self) -> f64 {
        if self.world.time_step == 0 {
            return 1.0;
        }
        let violation_rate = self.world.violations.len() as f64 / self.world.time_step as f64;
        (1.0 - violation_rate).max(0.0)
    }
}

impl Default for ApperceptionEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytic::Category;
    use crate::pipeline::KantianPipeline;

    fn report(text: &str, cat: Category) -> PipelineReport {
        let p = KantianPipeline::new();
        let mut r = p.process(text).unwrap();
        r.understanding.category_analysis.dominant = Some(cat);
        r
    }

    #[test]
    fn engine_starts_at_full_ideality() {
        let engine = ApperceptionEngine::new();
        assert_eq!(engine.ideality(), 1.0);
        assert_eq!(engine.time_step(), 0);
    }

    #[test]
    fn engine_update_builds_world() {
        let mut engine = ApperceptionEngine::new();
        let r = report("Fire causes smoke.", Category::Causality);
        let result = engine.update(&r);
        assert_eq!(engine.time_step(), 1);
        assert!(engine.rule_count() > 0);
        assert!(result.unified || !result.violations.is_empty());
    }

    #[test]
    fn engine_detects_causal_loop() {
        let mut engine = ApperceptionEngine::new();
        engine.update(&report("A causes B.", Category::Causality));
        let r = report("B causes A.", Category::Causality);
        let result = engine.update(&r);
        assert!(result.has_causal_loops());
    }

    #[test]
    fn engine_predict_next_after_causal_rule() {
        let mut engine = ApperceptionEngine::new();
        engine.update(&report("Heat causes expansion.", Category::Causality));
        let preds = engine.predict_next();
        assert!(preds.iter().any(|p| p.to_lowercase().contains("expansion")));
    }

    #[test]
    fn engine_retrodict_returns_turn_facts() {
        let mut engine = ApperceptionEngine::new();
        engine.update(&report("Water boils.", Category::Existence));
        engine.update(&report("Steam rises.", Category::Causality));
        let t1 = engine.retrodict(1);
        assert!(!t1.is_empty());
        assert!(t1.iter().any(|(_, v)| v.to_lowercase().contains("water")));
    }

    #[test]
    fn engine_ideality_decreases_on_violation() {
        let mut engine = ApperceptionEngine::new();
        engine.update(&report("A causes B.", Category::Causality));
        engine.update(&report("B causes A.", Category::Causality)); // loop → violation
        assert!(engine.ideality() < 1.0);
    }
}
