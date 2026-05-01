//! # MultiAgent Epistemic Bus (TRIZ B-1)
//!
//! Tracks epistemic states across multiple LLM agents and detects
//! cross-agent contradictions using Vepol triple comparison.
//!
//! ## Usage
//! ```rust,ignore
//! let mut bus = MultiAgentBus::new();
//! bus.register("agent-a", "Free will is real.").unwrap();
//! bus.register("agent-b", "Everything is causally determined and there is no free will.").unwrap();
//! let conflicts = bus.detect_conflicts();
//! assert!(!conflicts.is_empty());
//! ```

use crate::{
    dialectic::semantic_field::KeywordSemanticField,
    dialectic::AntinomyDetector,
    error::Result,
    pipeline::{KantianPipeline, PipelineReport, RiskLevel},
    types::Proposition,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A unique identifier for an agent in the bus.
pub type AgentId = String;

// ─── CrossAgentConflict ───────────────────────────────────────────────────────

/// An epistemic conflict detected between two agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossAgentConflict {
    /// The first agent involved.
    pub agent_a: AgentId,
    /// The second agent involved.
    pub agent_b: AgentId,
    /// Human-readable description of the conflict.
    pub description: String,
    /// Severity in [0.0, 1.0].
    pub severity: f64,
    /// The conflict kind.
    pub kind: ConflictKind,
}

/// Classification of cross-agent conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictKind {
    /// Both agents assert contradicting antinomial claims.
    AntinomialContradiction,
    /// One agent's output has HIGH risk while the other is SAFE.
    RiskDivergence,
    /// Both agents assert generic logical contradictions.
    LogicalContradiction,
}

// ─── MultiAgentBus ───────────────────────────────────────────────────────────

/// Bus that maintains epistemic state for multiple agents and detects conflicts.
pub struct MultiAgentBus {
    states: HashMap<AgentId, PipelineReport>,
    pipeline: KantianPipeline,
}

impl MultiAgentBus {
    /// Create a new empty bus.
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            pipeline: KantianPipeline::new(),
        }
    }

    /// Register text output from `agent_id`, running it through the pipeline.
    ///
    /// Returns a reference to the stored `PipelineReport`.
    pub fn register(
        &mut self,
        agent_id: impl Into<AgentId>,
        text: &str,
    ) -> Result<&PipelineReport> {
        let id = agent_id.into();
        let report = self.pipeline.process(text)?;
        self.states.insert(id.clone(), report);
        Ok(self.states.get(&id).unwrap())
    }

    /// Detect all epistemic conflicts between registered agents.
    pub fn detect_conflicts(&self) -> Vec<CrossAgentConflict> {
        let agents: Vec<(&AgentId, &PipelineReport)> = self.states.iter().collect();
        let mut conflicts = Vec::new();
        let field = KeywordSemanticField;

        for i in 0..agents.len() {
            for j in (i + 1)..agents.len() {
                let (id_a, report_a) = agents[i];
                let (id_b, report_b) = agents[j];

                // ── Antinomies: pool propositions from both agents ────────────
                let props_a = extract_propositions(report_a);
                let props_b = extract_propositions(report_b);

                let mut combined = props_a.clone();
                combined.extend(props_b.clone());

                let antinomies = AntinomyDetector::detect_with_field(&combined, &field);
                for ant in antinomies.iter().filter(|a| a.has_conflict) {
                    // Confirm the conflict spans agents (one prop from each side)
                    let thesis_from_a = ant
                        .thesis_proposition
                        .as_ref()
                        .map(|p| props_a.iter().any(|pa| pa.text == p.text))
                        .unwrap_or(false);
                    let antithesis_from_b = ant
                        .antithesis_proposition
                        .as_ref()
                        .map(|p| props_b.iter().any(|pb| pb.text == p.text))
                        .unwrap_or(false);
                    let thesis_from_b = ant
                        .thesis_proposition
                        .as_ref()
                        .map(|p| props_b.iter().any(|pb| pb.text == p.text))
                        .unwrap_or(false);
                    let antithesis_from_a = ant
                        .antithesis_proposition
                        .as_ref()
                        .map(|p| props_a.iter().any(|pa| pa.text == p.text))
                        .unwrap_or(false);

                    if (thesis_from_a && antithesis_from_b) || (thesis_from_b && antithesis_from_a)
                    {
                        conflicts.push(CrossAgentConflict {
                            agent_a: id_a.clone(),
                            agent_b: id_b.clone(),
                            description: ant.description.clone(),
                            severity: 1.0,
                            kind: ConflictKind::AntinomialContradiction,
                        });
                    }
                }

                // ── Risk divergence ───────────────────────────────────────────
                let risk_a = report_a.verdict.risk;
                let risk_b = report_b.verdict.risk;
                if risk_a != risk_b {
                    let severity = match (risk_a, risk_b) {
                        (RiskLevel::High, RiskLevel::Safe) | (RiskLevel::Safe, RiskLevel::High) => {
                            0.9
                        }
                        (RiskLevel::High, _) | (_, RiskLevel::High) => 0.7,
                        _ => 0.3,
                    };
                    conflicts.push(CrossAgentConflict {
                        agent_a: id_a.clone(),
                        agent_b: id_b.clone(),
                        description: format!(
                            "Risk level divergence: '{}' = {}, '{}' = {}",
                            id_a, risk_a, id_b, risk_b
                        ),
                        severity,
                        kind: ConflictKind::RiskDivergence,
                    });
                }
            }
        }

        // Sort by descending severity
        conflicts.sort_by(|a, b| {
            b.severity
                .partial_cmp(&a.severity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        conflicts
    }

    /// Get the pipeline report for a specific agent.
    pub fn agent_report(&self, agent_id: &str) -> Option<&PipelineReport> {
        self.states.get(agent_id)
    }

    /// List all registered agent IDs.
    pub fn agents(&self) -> Vec<&AgentId> {
        self.states.keys().collect()
    }

    /// Overall risk across all agents (highest wins).
    pub fn overall_risk(&self) -> RiskLevel {
        self.states
            .values()
            .map(|r| r.verdict.risk)
            .max()
            .unwrap_or(RiskLevel::Safe)
    }
}

impl Default for MultiAgentBus {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Extract all propositions (from antinomy reports) for a pipeline report.
fn extract_propositions(report: &PipelineReport) -> Vec<Proposition> {
    report
        .dialectic
        .antinomies
        .iter()
        .flat_map(|a| {
            let mut props = Vec::new();
            if let Some(p) = &a.thesis_proposition {
                props.push(p.clone());
            }
            if let Some(p) = &a.antithesis_proposition {
                props.push(p.clone());
            }
            props
        })
        .collect()
}

// ─── BusSummary ───────────────────────────────────────────────────────────────

/// A serializable summary of the bus state for CLI output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusSummary {
    pub agent_count: usize,
    pub conflict_count: usize,
    pub overall_risk: RiskLevel,
    pub conflicts: Vec<CrossAgentConflict>,
    pub agents: Vec<AgentSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSummary {
    pub id: String,
    pub risk: RiskLevel,
    pub has_illusions: bool,
    pub has_contradictions: bool,
}

impl MultiAgentBus {
    /// Generate a summary of the bus state.
    pub fn summarize(&self) -> BusSummary {
        let conflicts = self.detect_conflicts();
        let agents = self
            .states
            .iter()
            .map(|(id, r)| AgentSummary {
                id: id.clone(),
                risk: r.verdict.risk,
                has_illusions: r.verdict.has_illusions,
                has_contradictions: r.verdict.has_contradictions,
            })
            .collect();

        BusSummary {
            agent_count: self.states.len(),
            conflict_count: conflicts.len(),
            overall_risk: self.overall_risk(),
            conflicts,
            agents,
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_conflict_when_consistent() {
        let mut bus = MultiAgentBus::new();
        bus.register("a", "Water boils at 100 degrees.").unwrap();
        bus.register("b", "Ice melts at 0 degrees.").unwrap();
        let conflicts = bus.detect_conflicts();
        assert!(conflicts
            .iter()
            .all(|c| matches!(c.kind, ConflictKind::RiskDivergence)));
    }

    #[test]
    fn detects_antinomial_conflict() {
        let mut bus = MultiAgentBus::new();
        bus.register(
            "agent-a",
            "Humans have genuine free will and we are free agents.",
        )
        .unwrap();
        bus.register(
            "agent-b",
            "Everything is causally determined and there is no free will.",
        )
        .unwrap();
        let conflicts = bus.detect_conflicts();
        assert!(
            !conflicts.is_empty(),
            "Expected cross-agent antinomy conflict"
        );
    }

    #[test]
    fn overall_risk_max() {
        let mut bus = MultiAgentBus::new();
        bus.register("a", "Water is wet.").unwrap();
        bus.register("b", "God exists and is a necessary being.")
            .unwrap();
        let risk = bus.overall_risk();
        assert!(risk >= RiskLevel::Low);
    }
}
