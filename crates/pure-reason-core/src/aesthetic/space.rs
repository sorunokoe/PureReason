//! # Space — The Form of Outer Intuition
//!
//! For Kant, **Space** is the pure form of outer sense. It is not a concept
//! derived from experience, but the a priori framework within which all outer
//! appearances are organized.
//!
//! In this system, SpaceForm captures the **structural and relational organization**
//! of text: which entities appear, how they relate to each other, their syntactic
//! positions, and dependency structures.

use super::Manifold;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── SpatialRelation ─────────────────────────────────────────────────────────

/// Relations between structural nodes — the spatial "distances" in conceptual space.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpatialRelation {
    /// A is the subject of B (e.g., "water" is subject of "boils")
    SubjectOf,
    /// A is the object of B
    ObjectOf,
    /// A modifies B
    Modifies,
    /// A is coordinated with B (conjunction)
    CoordinatedWith,
    /// A contains B (part-whole)
    Contains,
    /// A is part of B
    PartOf,
    /// A is adjacent to B in sequence
    AdjacentTo,
    /// General relation with label
    RelatedTo(String),
}

// ─── StructuralNode ──────────────────────────────────────────────────────────

/// A node in the spatial structure of the text — an entity, concept, or term.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralNode {
    pub id: Uuid,
    pub text: String,
    pub position: usize,
    pub kind: NodeKind,
}

/// The grammatical/semantic kind of a structural node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Preposition,
    Other,
}

impl StructuralNode {
    pub fn new(text: impl Into<String>, position: usize, kind: NodeKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            text: text.into(),
            position,
            kind,
        }
    }
}

// ─── SpaceEdge ───────────────────────────────────────────────────────────────

/// A directed relation between two structural nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceEdge {
    pub from: Uuid,
    pub to: Uuid,
    pub relation: SpatialRelation,
}

// ─── SpaceForm ───────────────────────────────────────────────────────────────

/// The Form of Space — the structural/relational organization of input text.
///
/// This is a graph of structural nodes connected by spatial relations.
/// It captures "where" concepts appear relative to each other in the
/// conceptual space of the text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceForm {
    pub nodes: Vec<StructuralNode>,
    pub edges: Vec<SpaceEdge>,
}

impl SpaceForm {
    /// Organize a manifold into a spatial form.
    ///
    /// Extracts structural nodes from tokens and infers basic relations
    /// from positional adjacency and simple heuristics.
    pub fn organize(manifold: &Manifold) -> Result<Self> {
        let nodes = extract_nodes(&manifold.tokens);
        let edges = infer_edges(&nodes);
        Ok(Self { nodes, edges })
    }

    /// Find a node by its text.
    pub fn find_node(&self, text: &str) -> Option<&StructuralNode> {
        self.nodes.iter().find(|n| n.text == text)
    }

    /// Find all nodes of a given kind.
    pub fn nodes_of_kind(&self, kind: &NodeKind) -> Vec<&StructuralNode> {
        self.nodes.iter().filter(|n| &n.kind == kind).collect()
    }

    /// Find all edges from a given node.
    pub fn edges_from(&self, node_id: Uuid) -> Vec<&SpaceEdge> {
        self.edges.iter().filter(|e| e.from == node_id).collect()
    }
}

/// Simple heuristic: classify tokens into NodeKind based on suffix patterns.
/// A real implementation would use a POS tagger.
fn classify_token(token: &str) -> NodeKind {
    let t = token.to_lowercase();
    // Very simple heuristic based on common suffixes
    if t.ends_with("ly") {
        NodeKind::Adverb
    } else if t.ends_with("ing") || t.ends_with("ed") || t.ends_with("ize") || t.ends_with("ise") {
        NodeKind::Verb
    } else if t.ends_with("ful") || t.ends_with("less") || t.ends_with("ous") || t.ends_with("al") {
        NodeKind::Adjective
    } else if matches!(
        t.as_str(),
        "in" | "on" | "at" | "by" | "of" | "to" | "for" | "with" | "from" | "into"
    ) {
        NodeKind::Preposition
    } else {
        NodeKind::Noun
    }
}

fn extract_nodes(tokens: &[String]) -> Vec<StructuralNode> {
    tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| t.len() > 1) // skip single-char tokens
        .map(|(i, t)| {
            let kind = classify_token(t);
            StructuralNode::new(t.clone(), i, kind)
        })
        .collect()
}

fn infer_edges(nodes: &[StructuralNode]) -> Vec<SpaceEdge> {
    // For now: connect adjacent content nodes
    nodes
        .windows(2)
        .map(|pair| SpaceEdge {
            from: pair[0].id,
            to: pair[1].id,
            relation: SpatialRelation::AdjacentTo,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aesthetic::Manifold;

    #[test]
    fn space_form_extracts_nodes() {
        let m = Manifold::from_text("Water boils at 100 degrees.");
        let sf = SpaceForm::organize(&m).unwrap();
        assert!(!sf.nodes.is_empty());
    }

    #[test]
    fn space_form_has_edges() {
        let m = Manifold::from_text("The cat sat on the mat.");
        let sf = SpaceForm::organize(&m).unwrap();
        assert!(!sf.edges.is_empty());
    }
}
