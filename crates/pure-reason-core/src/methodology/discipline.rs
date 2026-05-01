//! # Discipline of Pure Reason
//!
//! The Discipline of Pure Reason is the negative component of the Methodology:
//! it prevents Pure Reason from overstepping its boundaries.
//!
//! Kant identifies several types of dogmatic overreach that the Discipline must restrain:
//! - Dogmatism: claiming certain metaphysical knowledge without critique
//! - Polemicism: engaging in endless metaphysical disputes
//! - Skepticism: claiming that knowledge is impossible in general
//! - Hypothetical use: using ideas of reason as if they were hypotheses about objects

use crate::types::Proposition;
use serde::{Deserialize, Serialize};

// ─── DisciplinaryRule ────────────────────────────────────────────────────────

/// A rule of the Discipline of Pure Reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisciplinaryRule {
    /// Do not make dogmatic metaphysical assertions beyond possible experience.
    NoDogmatism,
    /// Do not use transcendental ideas as if they were hypotheses about real objects.
    NoHypotheticalUseOfIdeas,
    /// Do not engage in empty polemic (endless disputes without possible resolution).
    NoEmptyPolemic,
    /// Do not fall into skepticism that denies all knowledge.
    NoGlobalSkepticism,
    /// Categories must only be applied within the domain of possible experience.
    CategoryBoundary,
    /// Reason must not pretend to prove or disprove what lies beyond experience.
    ExperienceBoundary,
}

impl DisciplinaryRule {
    pub fn name(&self) -> &'static str {
        match self {
            Self::NoDogmatism => "No Dogmatism",
            Self::NoHypotheticalUseOfIdeas => "No Hypothetical Use of Ideas",
            Self::NoEmptyPolemic => "No Empty Polemic",
            Self::NoGlobalSkepticism => "No Global Skepticism",
            Self::CategoryBoundary => "Category Boundary",
            Self::ExperienceBoundary => "Experience Boundary",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::NoDogmatism =>
                "Pure Reason must not make dogmatic assertions about things-in-themselves without critique.",
            Self::NoHypotheticalUseOfIdeas =>
                "Transcendental Ideas (Soul, World, God) must not be used as hypotheses about real objects.",
            Self::NoEmptyPolemic =>
                "Reason must not engage in endless polemical disputes where no resolution is possible.",
            Self::NoGlobalSkepticism =>
                "Reason must not deny all knowledge, since mathematics and natural science provide genuine knowledge.",
            Self::CategoryBoundary =>
                "The categories of the understanding (causality, substance, etc.) only apply within possible experience.",
            Self::ExperienceBoundary =>
                "Reason must acknowledge the boundary between the knowable (phenomena) and the unknowable (noumena).",
        }
    }

    pub fn violation_signals(&self) -> &'static [&'static str] {
        match self {
            Self::NoDogmatism => &[
                "it is absolutely certain that",
                "it is definitively proven that beyond doubt",
                "we know for certain that the ultimate nature",
                "the thing-in-itself is",
                "reality in itself is",
            ],
            Self::NoHypotheticalUseOfIdeas => &[
                "i hypothesize that the soul",
                "i assume god",
                "perhaps the world-totality",
                "let us suppose there is a necessary being",
            ],
            Self::NoEmptyPolemic => &[
                "this debate can never be resolved",
                "this question is meaningless",
                "it is pointless to ask",
            ],
            Self::NoGlobalSkepticism => &[
                "nothing can be known",
                "all knowledge is impossible",
                "we cannot know anything",
                "all is uncertain",
                "skepticism is the only position",
            ],
            Self::CategoryBoundary => &[
                "causality applies to god",
                "god is the cause of himself",
                "the noumenon has properties",
                "things-in-themselves are caused by",
            ],
            Self::ExperienceBoundary => &[
                "we can know things-in-themselves",
                "noumena can be experienced",
                "the thing in itself is knowable",
                "we perceive reality as it is in itself",
            ],
        }
    }
}

// ─── ViolationReport ─────────────────────────────────────────────────────────

/// A detected violation of a disciplinary rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationReport {
    pub rule: DisciplinaryRule,
    pub proposition: Proposition,
    pub trigger: String,
    pub correction: String,
}

// ─── Discipline ──────────────────────────────────────────────────────────────

/// The Discipline of Pure Reason — enforces boundaries on rational claims.
pub struct Discipline;

impl Discipline {
    pub fn new() -> Self {
        Self
    }

    /// Check propositions for disciplinary violations.
    pub fn check(&self, propositions: &[Proposition]) -> Vec<ViolationReport> {
        let rules = [
            DisciplinaryRule::NoDogmatism,
            DisciplinaryRule::NoHypotheticalUseOfIdeas,
            DisciplinaryRule::NoEmptyPolemic,
            DisciplinaryRule::NoGlobalSkepticism,
            DisciplinaryRule::CategoryBoundary,
            DisciplinaryRule::ExperienceBoundary,
        ];

        let mut violations = Vec::new();

        for prop in propositions {
            let text = prop.text.to_lowercase();

            for rule in &rules {
                for &signal in rule.violation_signals() {
                    if text.contains(signal) {
                        violations.push(ViolationReport {
                            rule: *rule,
                            proposition: prop.clone(),
                            trigger: signal.to_string(),
                            correction: format!(
                                "Rule violated: {}. {}",
                                rule.name(),
                                rule.description()
                            ),
                        });
                        break;
                    }
                }
            }
        }

        violations
    }
}

impl Default for Discipline {
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
    fn skepticism_detected() {
        let p = prop("Nothing can be known and all knowledge is impossible");
        let d = Discipline::new();
        let violations = d.check(&[p]);
        assert!(violations
            .iter()
            .any(|v| v.rule == DisciplinaryRule::NoGlobalSkepticism));
    }

    #[test]
    fn no_violations_in_clean_text() {
        let p = prop("Water is composed of hydrogen and oxygen");
        let d = Discipline::new();
        let violations = d.check(&[p]);
        assert!(violations.is_empty());
    }
}
