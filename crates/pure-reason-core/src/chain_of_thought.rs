//! Chain of Thought Reasoning: Explicit reasoning step extraction and validation
//!
//! TRIZ Principle: Transition to Micro-Level + Feedback
//! Make reasoning visible at the structural level, showing each logical step.
//!
//! This module enables transparent reasoning chains where each premise is validated
//! independently before concluding. Inspired by o3's approach but deterministic.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tracing::info;

/// A single reasoning step in a chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    /// Step number (0 = initial claim, 1+ = intermediate conclusions)
    pub step_index: usize,
    /// The statement at this step
    pub statement: String,
    /// Type of step: Premise, Inference, Conclusion
    pub step_type: StepType,
    /// Confidence in this step (0.0-1.0)
    pub confidence: f64,
    /// Reasoning for this step
    pub reasoning: String,
    /// Dependencies: indices of steps this depends on
    pub depends_on: Vec<usize>,
    /// Strongest supporting evidence
    pub evidence: Vec<String>,
}

/// Type of reasoning step
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepType {
    /// Initial premise or given fact
    Premise,
    /// Intermediate logical inference
    Inference,
    /// Final conclusion
    Conclusion,
}

impl std::fmt::Display for StepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepType::Premise => write!(f, "Premise"),
            StepType::Inference => write!(f, "Inference"),
            StepType::Conclusion => write!(f, "Conclusion"),
        }
    }
}

/// Complete reasoning chain for a claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningChain {
    /// Original claim being evaluated
    pub claim: String,
    /// All steps in the reasoning chain
    pub steps: Vec<ReasoningStep>,
    /// Weakest confidence in the chain (bottleneck)
    pub weakest_step_confidence: f64,
    /// Overall chain validity (product of all confidences)
    pub chain_validity: f64,
    /// Weakest step index (where confidence is lowest)
    pub weakest_step_index: usize,
}

impl ReasoningChain {
    /// Create a new reasoning chain from steps
    pub fn new(claim: String, mut steps: Vec<ReasoningStep>) -> Self {
        // Ensure conclusion is last
        if let Some(conclusion_idx) = steps
            .iter()
            .position(|s| s.step_type == StepType::Conclusion)
        {
            let conclusion = steps.remove(conclusion_idx);
            steps.push(conclusion);
        }

        // Calculate chain validity
        let weakest_confidence = steps.iter().map(|s| s.confidence).fold(1.0, f64::min);
        let chain_validity = steps.iter().map(|s| s.confidence).fold(1.0, |a, b| a * b);

        let weakest_step_index = steps
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.confidence.partial_cmp(&b.1.confidence).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);

        Self {
            claim,
            steps,
            weakest_step_confidence: weakest_confidence,
            chain_validity,
            weakest_step_index,
        }
    }

    /// Get summary of the chain
    pub fn summary(&self) -> String {
        format!(
            "Chain: {} steps, validity={:.2}, weakest={:.2} at step {}",
            self.steps.len(),
            self.chain_validity,
            self.weakest_step_confidence,
            self.weakest_step_index
        )
    }

    /// Get step dependencies as a graph (for visualization)
    pub fn dependency_graph(&self) -> HashMap<usize, Vec<usize>> {
        let mut graph = HashMap::new();
        for step in &self.steps {
            graph.insert(step.step_index, step.depends_on.clone());
        }
        graph
    }

    /// Check if chain is valid (all steps have confidence > threshold)
    pub fn is_valid(&self, confidence_threshold: f64) -> bool {
        self.steps
            .iter()
            .all(|step| step.confidence >= confidence_threshold)
    }

    /// Get the critical path (chain of dependencies to final conclusion)
    pub fn critical_path(&self) -> Vec<usize> {
        let mut path = vec![];
        let mut queue: VecDeque<usize> = VecDeque::new();

        // Start from conclusion
        if let Some(conclusion) = self
            .steps
            .iter()
            .position(|s| s.step_type == StepType::Conclusion)
        {
            queue.push_back(conclusion);
        }

        while let Some(step_idx) = queue.pop_front() {
            if !path.contains(&step_idx) {
                path.push(step_idx);
                if let Some(step) = self.steps.get(step_idx) {
                    for dep in &step.depends_on {
                        if !path.contains(dep) {
                            queue.push_back(*dep);
                        }
                    }
                }
            }
        }

        path
    }

    /// Get textual explanation of reasoning chain
    pub fn explain(&self) -> String {
        let mut explanation = format!("**Claim**: {}\n\n", self.claim);
        explanation.push_str("**Reasoning Steps**:\n");

        for (idx, step) in self.steps.iter().enumerate() {
            explanation.push_str(&format!(
                "{}. [{}] {} (confidence: {:.0}%)\n   Reasoning: {}\n",
                idx,
                step.step_type,
                step.statement,
                step.confidence * 100.0,
                step.reasoning
            ));

            if !step.evidence.is_empty() {
                explanation.push_str("   Evidence:\n");
                for ev in &step.evidence {
                    explanation.push_str(&format!("   - {}\n", ev));
                }
            }

            if !step.depends_on.is_empty() {
                explanation.push_str(&format!("   Depends on: steps {:?}\n", step.depends_on));
            }
        }

        explanation.push_str(&format!(
            "\n**Chain Validity**: {:.1}% (bottleneck: step {} with {:.0}%)\n",
            self.chain_validity * 100.0,
            self.weakest_step_index,
            self.weakest_step_confidence * 100.0
        ));

        explanation
    }
}

/// Builder for constructing reasoning chains
pub struct ChainBuilder {
    claim: String,
    steps: Vec<ReasoningStep>,
}

impl ChainBuilder {
    /// Create a new chain builder
    pub fn new(claim: String) -> Self {
        Self {
            claim,
            steps: vec![],
        }
    }

    /// Add a premise
    pub fn premise(mut self, statement: String, confidence: f64, evidence: Vec<String>) -> Self {
        self.steps.push(ReasoningStep {
            step_index: self.steps.len(),
            statement,
            step_type: StepType::Premise,
            confidence,
            reasoning: "Given fact or assumption".to_string(),
            depends_on: vec![],
            evidence,
        });
        self
    }

    /// Add an inference step
    pub fn inference(
        mut self,
        statement: String,
        confidence: f64,
        reasoning: String,
        depends_on: Vec<usize>,
        evidence: Vec<String>,
    ) -> Self {
        self.steps.push(ReasoningStep {
            step_index: self.steps.len(),
            statement,
            step_type: StepType::Inference,
            confidence,
            reasoning,
            depends_on,
            evidence,
        });
        self
    }

    /// Add a conclusion
    pub fn conclusion(
        mut self,
        statement: String,
        confidence: f64,
        reasoning: String,
        depends_on: Vec<usize>,
        evidence: Vec<String>,
    ) -> Self {
        self.steps.push(ReasoningStep {
            step_index: self.steps.len(),
            statement,
            step_type: StepType::Conclusion,
            confidence,
            reasoning,
            depends_on,
            evidence,
        });
        self
    }

    /// Build the chain
    pub fn build(self) -> ReasoningChain {
        info!("Built reasoning chain with {} steps", self.steps.len());
        ReasoningChain::new(self.claim, self.steps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reasoning_step_creation() {
        let step = ReasoningStep {
            step_index: 0,
            statement: "X is true".to_string(),
            step_type: StepType::Premise,
            confidence: 0.95,
            reasoning: "Given".to_string(),
            depends_on: vec![],
            evidence: vec!["Evidence 1".to_string()],
        };
        assert_eq!(step.confidence, 0.95);
    }

    #[test]
    fn test_chain_builder() {
        let chain = ChainBuilder::new("Vaccine is safe".to_string())
            .premise(
                "Clinical trials passed".to_string(),
                0.95,
                vec!["FDA data".to_string()],
            )
            .premise("No serious side effects reported".to_string(), 0.90, vec![])
            .inference(
                "Evidence suggests safety".to_string(),
                0.92,
                "From premises 0 and 1".to_string(),
                vec![0, 1],
                vec!["Meta-analysis".to_string()],
            )
            .conclusion(
                "Vaccine is safe".to_string(),
                0.92,
                "Follows from inference".to_string(),
                vec![2],
                vec![],
            )
            .build();

        assert_eq!(chain.steps.len(), 4);
        assert!(chain.is_valid(0.85));
    }

    #[test]
    fn test_chain_validity_calculation() {
        let chain = ChainBuilder::new("Test".to_string())
            .premise("P1".to_string(), 0.80, vec![])
            .premise("P2".to_string(), 0.90, vec![])
            .conclusion(
                "Conclusion".to_string(),
                0.85,
                "From P1 and P2".to_string(),
                vec![0, 1],
                vec![],
            )
            .build();

        assert_eq!(chain.weakest_step_confidence, 0.80);
        assert!(chain.chain_validity < 1.0);
    }

    #[test]
    fn test_chain_explain() {
        let chain = ChainBuilder::new("Test claim".to_string())
            .premise("Premise 1".to_string(), 0.95, vec!["Evidence".to_string()])
            .conclusion(
                "Conclusion".to_string(),
                0.95,
                "From premise".to_string(),
                vec![0],
                vec![],
            )
            .build();

        let explanation = chain.explain();
        assert!(explanation.contains("Claim"));
        assert!(explanation.contains("Reasoning Steps"));
        assert!(explanation.contains("95%"));
    }

    #[test]
    fn test_dependency_graph() {
        let chain = ChainBuilder::new("Test".to_string())
            .premise("P1".to_string(), 0.90, vec![])
            .premise("P2".to_string(), 0.90, vec![])
            .inference(
                "I1".to_string(),
                0.85,
                "From P1 and P2".to_string(),
                vec![0, 1],
                vec![],
            )
            .build();

        let graph = chain.dependency_graph();
        assert_eq!(graph.len(), 3);
        assert_eq!(graph.get(&2), Some(&vec![0, 1]));
    }

    #[test]
    fn test_step_type_display() {
        assert_eq!(format!("{}", StepType::Premise), "Premise");
        assert_eq!(format!("{}", StepType::Inference), "Inference");
        assert_eq!(format!("{}", StepType::Conclusion), "Conclusion");
    }

    #[test]
    fn test_chain_summary() {
        let chain = ChainBuilder::new("Test".to_string())
            .premise("P1".to_string(), 0.90, vec![])
            .conclusion(
                "C1".to_string(),
                0.90,
                "From P1".to_string(),
                vec![0],
                vec![],
            )
            .build();

        let summary = chain.summary();
        assert!(summary.contains("2 steps"));
        assert!(summary.contains("validity"));
    }

    #[test]
    fn test_critical_path() {
        let chain = ChainBuilder::new("Test".to_string())
            .premise("P1".to_string(), 0.90, vec![])
            .premise("P2".to_string(), 0.90, vec![])
            .inference(
                "I1".to_string(),
                0.85,
                "From P1".to_string(),
                vec![0],
                vec![],
            )
            .inference(
                "I2".to_string(),
                0.85,
                "From I1 and P2".to_string(),
                vec![2, 1],
                vec![],
            )
            .conclusion(
                "C1".to_string(),
                0.85,
                "From I2".to_string(),
                vec![3],
                vec![],
            )
            .build();

        let path = chain.critical_path();
        assert!(!path.is_empty());
        assert!(path.contains(&4)); // Conclusion
    }
}
