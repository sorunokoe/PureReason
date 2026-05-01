//! # Architectonic of Pure Reason
//!
//! The Architectonic is Kant's term for the art of constructing systems.
//! A system (for Kant) is "the unity of the manifold cognitions under one idea."
//!
//! The Architectonic chapter describes how the entire Critique of Pure Reason
//! is itself a system — organized by the idea of pure reason — and how genuine
//! science must always be systematic rather than merely aggregative.
//!
//! In our implementation, the Architectonic provides the system's self-description:
//! its structure, its unity, and its place in the larger system of knowledge.

use serde::{Deserialize, Serialize};

// ─── ComponentDescription ────────────────────────────────────────────────────

/// Description of one component of the PureReason system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDescription {
    pub name: String,
    pub role: String,
    pub kant_reference: String,
}

// ─── SystemDescription ───────────────────────────────────────────────────────

/// The self-description of the PureReason system as a unified whole.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDescription {
    pub title: String,
    pub unifying_idea: String,
    pub components: Vec<ComponentDescription>,
    pub purpose: String,
    pub limits: String,
}

// ─── Architectonic ───────────────────────────────────────────────────────────

/// The Architectonic of Pure Reason — the system's self-reflection.
pub struct Architectonic;

impl Architectonic {
    pub fn new() -> Self {
        Self
    }

    /// Produce a self-description of the PureReason system.
    pub fn describe(&self) -> SystemDescription {
        SystemDescription {
            title: "PureReason — Kant's Critique of Pure Reason as a Reasoning System".to_string(),
            unifying_idea: "The systematic unity of all possible experience under the conditions \
                           of pure intuition, pure understanding, and practical reason, with \
                           Wittgensteinian grounding for language and rule use.".to_string(),
            components: vec![
                ComponentDescription {
                    name: "Transcendental Aesthetic".to_string(),
                    role: "Processes raw input into structured intuitions via the forms of Space and Time".to_string(),
                    kant_reference: "CPR, Part I: Transcendental Doctrine of Elements, First Part".to_string(),
                },
                ComponentDescription {
                    name: "Transcendental Analytic: Categories".to_string(),
                    role: "Applies the 12 pure categories of the understanding to structure conceptual content".to_string(),
                    kant_reference: "CPR, Analytic of Concepts, §§10-12 (Table of Categories)".to_string(),
                },
                ComponentDescription {
                    name: "Transcendental Analytic: Schematism".to_string(),
                    role: "Bridges pure categories to intuition via temporal determinations (schemas)".to_string(),
                    kant_reference: "CPR, Analytic of Principles, Chapter I: Schematism".to_string(),
                },
                ComponentDescription {
                    name: "Transcendental Analytic: Principles".to_string(),
                    role: "The highest rules of the understanding: Axioms, Anticipations, Analogies, Postulates".to_string(),
                    kant_reference: "CPR, Analytic of Principles, Chapter II: System of All Principles".to_string(),
                },
                ComponentDescription {
                    name: "Transcendental Dialectic: Ideas".to_string(),
                    role: "The three Transcendental Ideas (Soul, World, God) in their regulative use".to_string(),
                    kant_reference: "CPR, Part II: Transcendental Dialectic, Introduction".to_string(),
                },
                ComponentDescription {
                    name: "Transcendental Dialectic: Paralogisms".to_string(),
                    role: "Detects invalid self-referential reasoning about the thinking subject".to_string(),
                    kant_reference: "CPR, Dialectic, Book II, Chapter I: Paralogisms of Pure Reason".to_string(),
                },
                ComponentDescription {
                    name: "Transcendental Dialectic: Antinomies".to_string(),
                    role: "Detects contradictory claims about the World-totality".to_string(),
                    kant_reference: "CPR, Dialectic, Book II, Chapter II: Antinomy of Pure Reason".to_string(),
                },
                ComponentDescription {
                    name: "Transcendental Dialectic: Ideal".to_string(),
                    role: "Analyzes theological arguments and their legitimate regulative interpretation".to_string(),
                    kant_reference: "CPR, Dialectic, Book II, Chapter III: Ideal of Pure Reason".to_string(),
                },
                ComponentDescription {
                    name: "Transcendental Methodology: Discipline".to_string(),
                    role: "Enforces boundaries preventing dogmatic overreach".to_string(),
                    kant_reference: "CPR, Transcendental Doctrine of Method, Chapter I: Discipline".to_string(),
                },
                ComponentDescription {
                    name: "Transcendental Methodology: Canon".to_string(),
                    role: "Identifies legitimate practical and regulative uses of reason".to_string(),
                    kant_reference: "CPR, Transcendental Doctrine of Method, Chapter II: Canon".to_string(),
                },
                ComponentDescription {
                    name: "Wittgensteinian Layer: Language Games".to_string(),
                    role: "Detects context/domain (language game) of interaction for contextual grounding".to_string(),
                    kant_reference: "Wittgenstein, Philosophical Investigations §§1-137".to_string(),
                },
                ComponentDescription {
                    name: "Wittgensteinian Layer: Tractatus".to_string(),
                    role: "Structured fact representation, speakable/showable boundary".to_string(),
                    kant_reference: "Wittgenstein, Tractatus Logico-Philosophicus (1921)".to_string(),
                },
                ComponentDescription {
                    name: "LLM Integration".to_string(),
                    role: "Provider-agnostic LLM interface; KantianAgent wraps LLMs with the full pipeline".to_string(),
                    kant_reference: "Applied system integration (non-Kantian; practical implementation layer)".to_string(),
                },
            ],
            purpose: "To provide a structured reasoning and validation framework for LLMs, \
                     grounded in Kant's epistemology. The system can analyze text for its \
                     categorical structure, detect dialectical illusions (analogous to hallucinations), \
                     validate claims against the principles of experience, and provide a philosophical \
                     foundation for epistemic humility in artificial reasoning.".to_string(),
            limits: "The system operates on natural language heuristics, not formal logic proofs. \
                    Category detection and illusion detection are probabilistic, not certain. \
                    The system cannot itself transcend the phenomenal/noumenal boundary it describes — \
                    it analyzes text as given, not things-in-themselves.".to_string(),
        }
    }
}

impl Default for Architectonic {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn description_has_all_components() {
        let arch = Architectonic::new();
        let desc = arch.describe();
        assert!(desc.components.len() >= 10);
        assert!(!desc.title.is_empty());
        assert!(!desc.unifying_idea.is_empty());
    }
}
