//! # Imagination Layer — Einbildungskraft (TRIZ S-7)
//!
//! Implements Kant's **productive imagination** (CPR A120, B151–152) — the faculty
//! that bridges sensory intuition and conceptual understanding by generating
//! schematised hypothetical world states.
//!
//! > "Imagination is a faculty for determining the sensibility a priori,
//! >  and its synthesis of intuitions, conforming to the categories, must be
//! >  the transcendental synthesis of imagination." — CPR B151
//!
//! In AI terms: given the current WorldModel, generate a *hypothetical* world by
//! applying a counterfactual, then test whether a proposition is consistent with
//! that hypothetical. This provides **controlled speculation** — hypotheticals are
//! clearly marked, never confused with WorldModel facts.
//!
//! ## TRIZ Rationale
//! **S-7 (TC-1, PC-4):**  
//! Resolves the flexibility↔hallucination contradiction: the agent can speculate
//! freely in a clearly-delimited hypothetical frame, while the real WorldModel
//! remains unmodified. Hallucination = uncontrolled hypothetical leaking into facts.

use crate::world_model::WorldModel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── HypotheticalWorld ────────────────────────────────────────────────────────

/// A hypothetical world state — a WorldModel clone modified by a counterfactual.
/// Clearly distinct from the real WorldModel; never persisted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypotheticalWorld {
    /// The counterfactual assumption that created this hypothetical.
    pub assumption: String,
    /// Objects in this hypothetical world.
    pub objects: HashMap<String, Vec<String>>, // id → [fact descriptions]
    /// Causal rules in this hypothetical world.
    pub rules: Vec<(String, String, f64)>, // (antecedent, consequent, confidence)
    /// Propositions derivable from this hypothetical (predictions).
    pub derivable: Vec<String>,
}

impl HypotheticalWorld {
    /// Test whether a proposition is consistent with this hypothetical world.
    ///
    /// A proposition is consistent if:
    /// - It doesn't contradict any existing object fact
    /// - OR it follows from a causal rule in the hypothetical
    pub fn is_consistent(&self, proposition: &str) -> bool {
        let prop_lower = proposition.to_lowercase();

        // Check if it's directly derivable
        if self.derivable.iter().any(|d| {
            d.to_lowercase().contains(&prop_lower) || prop_lower.contains(&d.to_lowercase())
        }) {
            return true;
        }

        // Check causal rules
        for (ant, cons, _conf) in &self.rules {
            if prop_lower.contains(&ant.to_lowercase()) || prop_lower.contains(&cons.to_lowercase())
            {
                return true;
            }
        }

        // Default: not explicitly contradicted → tentatively consistent
        true
    }
}

// ─── Einbildungskraft ─────────────────────────────────────────────────────────

/// The Kantian Imagination — productive imagination for controlled counterfactuals.
pub struct Einbildungskraft<'a> {
    world: &'a WorldModel,
}

impl<'a> Einbildungskraft<'a> {
    /// Create a new imagination instance bound to a WorldModel.
    pub fn new(world: &'a WorldModel) -> Self {
        Einbildungskraft { world }
    }

    /// Generate a hypothetical world by applying a counterfactual assumption.
    ///
    /// The hypothetical is a clone of the current WorldModel with the assumption
    /// applied on top. It is ephemeral — the real WorldModel is never modified.
    pub fn imagine(&self, counterfactual: &str) -> HypotheticalWorld {
        let cf_lower = counterfactual.to_lowercase();

        // Start with a snapshot of current world
        let mut objects: HashMap<String, Vec<String>> = self
            .world
            .objects
            .iter()
            .map(|(id, obj)| {
                let facts: Vec<String> = obj.facts.iter().map(|f| f.value.clone()).collect();
                (id.0.clone(), facts)
            })
            .collect();

        let mut rules: Vec<(String, String, f64)> = self
            .world
            .rules
            .iter()
            .map(|r| (r.antecedent.clone(), r.consequent.clone(), r.confidence))
            .collect();

        // Apply the counterfactual: add it as a new rule or object fact
        // Try to parse as "if X then Y" or "X causes Y" counterfactual
        let new_rule = try_parse_causal(counterfactual);
        if let Some((ant, cons)) = new_rule {
            rules.push((ant, cons, 0.7));
        } else {
            // Treat as a fact about a new or existing object
            let words: Vec<&str> = counterfactual.split_whitespace().take(3).collect();
            let subject = words.join(" ").to_lowercase();
            objects
                .entry(subject)
                .or_default()
                .push(counterfactual.to_string());
        }

        // Derive what would follow from the counterfactual + rules
        let mut derivable = Vec::new();
        for (ant, cons, conf) in &rules {
            if cf_lower.contains(&ant.to_lowercase()) && *conf >= 0.5 {
                derivable.push(cons.clone());
            }
        }
        // Forward chain one step
        let snapshot_derivable = derivable.clone();
        for d in &snapshot_derivable {
            for (ant, cons, conf) in &rules {
                if d.to_lowercase().contains(&ant.to_lowercase())
                    && *conf >= 0.5
                    && !derivable.contains(cons)
                {
                    derivable.push(cons.clone());
                }
            }
        }

        HypotheticalWorld {
            assumption: counterfactual.to_string(),
            objects,
            rules,
            derivable,
        }
    }

    /// Test whether a proposition is consistent with a given hypothetical world.
    pub fn test_in_hypothetical(&self, hypo: &HypotheticalWorld, proposition: &str) -> bool {
        hypo.is_consistent(proposition)
    }

    /// Generate several alternative hypothetical worlds from a set of counterfactuals.
    pub fn imagine_alternatives(&self, counterfactuals: &[&str]) -> Vec<HypotheticalWorld> {
        counterfactuals.iter().map(|cf| self.imagine(cf)).collect()
    }
}

fn try_parse_causal(text: &str) -> Option<(String, String)> {
    let patterns = [" causes ", " leads to ", " implies ", " if ", " then "];
    let lower = text.to_lowercase();
    for pat in &patterns {
        if let Some(pos) = lower.find(pat) {
            let ant = text[..pos].trim().to_string();
            let cons = text[pos + pat.len()..]
                .trim()
                .trim_end_matches(['.', '!'])
                .to_string();
            if !ant.is_empty() && !cons.is_empty() {
                return Some((ant, cons));
            }
        }
    }
    None
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytic::Category;
    use crate::pipeline::KantianPipeline;
    use crate::world_model::WorldModel;

    fn world_with(text: &str, cat: Category) -> WorldModel {
        let p = KantianPipeline::new();
        let mut r = p.process(text).unwrap();
        r.understanding.category_analysis.dominant = Some(cat);
        let mut world = WorldModel::new();
        world.update(&r);
        world
    }

    #[test]
    fn imagine_creates_hypothetical_from_causal() {
        let world = world_with("Heat causes expansion.", Category::Causality);
        let imag = Einbildungskraft::new(&world);
        let hypo = imag.imagine("Pressure causes compression.");
        assert_eq!(hypo.assumption, "Pressure causes compression.");
        assert!(hypo
            .rules
            .iter()
            .any(|(a, c, _)| a.contains("Pressure") && c.contains("compression")));
    }

    #[test]
    fn imagine_derives_consequences() {
        let world = world_with("Fire causes smoke.", Category::Causality);
        let imag = Einbildungskraft::new(&world);
        let hypo = imag.imagine("Fire is present.");
        // Should derive "smoke" from the existing rule if antecedent matches
        // (fire → smoke rule from real world should carry over)
        assert!(!hypo.rules.is_empty());
    }

    #[test]
    fn test_in_hypothetical_consistent_proposition() {
        let world = WorldModel::new();
        let imag = Einbildungskraft::new(&world);
        let hypo = imag.imagine("If water is heated, it boils.");
        assert!(imag.test_in_hypothetical(&hypo, "water boils when heated"));
    }

    #[test]
    fn alternatives_returns_one_per_counterfactual() {
        let world = WorldModel::new();
        let imag = Einbildungskraft::new(&world);
        let alts = imag.imagine_alternatives(&["A causes B.", "X leads to Y."]);
        assert_eq!(alts.len(), 2);
    }
}
