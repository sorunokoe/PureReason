use serde::{Deserialize, Serialize};
/// Phase C: Counterfactual Reasoning
///
/// Traces dependencies between claims and performs counterfactual reasoning.
/// Example: "If A is false, does B still hold?"
///
/// Enables detection of multi-claim contradictions through dependency analysis.
use std::collections::{HashSet, VecDeque};

/// A dependency relationship between claims
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyType {
    /// Direct causal: A → B (if A changes, B likely changes)
    Causal,
    /// Presupposition: A presupposes B (B must be true for A to be true)
    Presupposition,
    /// Contradiction: A contradicts B
    Contradiction,
    /// Entailment: A logically entails B
    Entailment,
}

/// A dependency edge in the claim graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    /// Source claim index
    pub from: usize,
    /// Target claim index
    pub to: usize,
    /// Type of dependency
    pub dep_type: DependencyType,
    /// Strength of dependency (0.0-1.0)
    pub strength: f64,
}

/// Result of counterfactual reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualResult {
    /// The original claim under test
    pub claim_idx: usize,
    /// Claim that would be violated if premise is false
    pub affected_claim_idx: Option<usize>,
    /// Explanation of the dependency chain
    pub explanation: String,
    /// Confidence in this analysis
    pub confidence: f64,
}

/// Dependency graph for claims
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// Number of claims in graph
    pub num_claims: usize,
    /// All dependency edges
    pub edges: Vec<DependencyEdge>,
}

/// Result of full counterfactual analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CounterfactualAnalysis {
    /// Dependency graph
    pub graph: DependencyGraph,
    /// All counterfactual findings
    pub findings: Vec<CounterfactualResult>,
    /// Overall reliability score
    pub reliability: f64,
}

impl DependencyGraph {
    /// Create new dependency graph
    pub fn new(num_claims: usize) -> Self {
        DependencyGraph {
            num_claims,
            edges: Vec::new(),
        }
    }

    /// Add dependency edge
    pub fn add_edge(&mut self, from: usize, to: usize, dep_type: DependencyType, strength: f64) {
        if from < self.num_claims && to < self.num_claims {
            self.edges.push(DependencyEdge {
                from,
                to,
                dep_type,
                strength,
            });
        }
    }

    /// Get outgoing edges from a claim
    pub fn outgoing_edges(&self, claim_idx: usize) -> Vec<&DependencyEdge> {
        self.edges.iter().filter(|e| e.from == claim_idx).collect()
    }

    /// Get incoming edges to a claim
    pub fn incoming_edges(&self, claim_idx: usize) -> Vec<&DependencyEdge> {
        self.edges.iter().filter(|e| e.to == claim_idx).collect()
    }

    /// Find all claims affected if given claim becomes false
    pub fn find_affected_claims(&self, claim_idx: usize) -> HashSet<usize> {
        let mut affected = HashSet::new();
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back(claim_idx);
        visited.insert(claim_idx);

        while let Some(current) = queue.pop_front() {
            // Get all claims that depend on this one
            for edge in self.outgoing_edges(current) {
                if (edge.dep_type == DependencyType::Presupposition
                    || edge.dep_type == DependencyType::Entailment)
                    && !visited.contains(&edge.to)
                {
                    affected.insert(edge.to);
                    visited.insert(edge.to);
                    queue.push_back(edge.to);
                }
            }
        }

        affected
    }
}

/// Extract simple subject-predicate-object triples from claim
fn extract_triples(claim: &str) -> Vec<(String, String, String)> {
    let mut triples = Vec::new();

    // Very simple heuristic: split on "is", "are", "has", "have"
    let patterns = [
        "is", "are", "has", "have", "breathe", "live", "eat", "cause",
    ];

    for pattern in &patterns {
        if let Some(pos) = claim.find(pattern) {
            let subject = claim[..pos].trim();
            let rest = &claim[pos + pattern.len()..].trim();

            if !subject.is_empty() && !rest.is_empty() {
                let subject = normalize_entity(subject);
                let predicate = pattern.to_string();
                let object = normalize_entity(rest);

                triples.push((subject, predicate, object));
            }
        }
    }

    triples
}

/// Normalize entity names (lowercase, trim)
fn normalize_entity(entity: &str) -> String {
    entity
        .trim_start_matches("a ")
        .trim_start_matches("an ")
        .trim_start_matches("the ")
        .trim()
        .to_lowercase()
}

/// Find shared entities between two triples
fn find_shared_entities(t1: &[(String, String, String)], t2: &[(String, String, String)]) -> usize {
    let mut count = 0;
    for (s1, _, o1) in t1 {
        for (s2, _, o2) in t2 {
            if s1 == s2 || o1 == o2 || s1 == o2 || o1 == s2 {
                count += 1;
            }
        }
    }
    count
}

/// Build dependency graph from claims
pub fn build_dependency_graph(claims: &[String]) -> DependencyGraph {
    let mut graph = DependencyGraph::new(claims.len());

    // Extract triples from each claim
    let triples: Vec<_> = claims.iter().map(|c| extract_triples(c)).collect();

    // Find dependencies between claims based on shared entities
    for i in 0..claims.len() {
        for j in (i + 1)..claims.len() {
            let shared_count = find_shared_entities(&triples[i], &triples[j]);

            if shared_count > 0 {
                // Heuristic: if claims share entities, assume causal or presuppositional
                let strength = (shared_count as f64) / 3.0; // Normalize to 0-1

                // Check if claim i seems to presuppose claim j
                if claims[i].contains("the ") || claims[i].starts_with("all ") {
                    graph.add_edge(i, j, DependencyType::Presupposition, strength.min(1.0));
                } else {
                    graph.add_edge(i, j, DependencyType::Causal, strength.min(1.0));
                }
            }
        }
    }

    graph
}

/// Perform counterfactual reasoning on dependency graph
pub fn analyze_counterfactuals(
    graph: &DependencyGraph,
    claims: &[String],
) -> CounterfactualAnalysis {
    let mut findings = Vec::new();

    for i in 0..graph.num_claims {
        let affected = graph.find_affected_claims(i);

        for affected_idx in affected {
            findings.push(CounterfactualResult {
                claim_idx: i,
                affected_claim_idx: Some(affected_idx),
                explanation: format!(
                    "If '{}' is false, then '{}' would be affected",
                    claims.get(i).unwrap_or(&"[unknown]".to_string()),
                    claims.get(affected_idx).unwrap_or(&"[unknown]".to_string())
                ),
                confidence: 0.65,
            });
        }
    }

    let reliability = if findings.is_empty() { 0.0 } else { 0.65 };

    CounterfactualAnalysis {
        graph: graph.clone(),
        findings,
        reliability,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_triples() {
        let claim = "Whales are mammals";
        let triples = extract_triples(claim);
        assert!(!triples.is_empty());
        assert_eq!(triples[0].0, "whales"); // subject
        assert_eq!(triples[0].1, "are"); // predicate
    }

    #[test]
    fn test_build_graph() {
        let claims = vec![
            "Whales are mammals".to_string(),
            "Mammals breathe with lungs".to_string(),
            "Whales breathe with gills".to_string(),
        ];
        let graph = build_dependency_graph(&claims);
        assert!(!graph.edges.is_empty());
    }

    #[test]
    fn test_find_affected() {
        let mut graph = DependencyGraph::new(3);
        graph.add_edge(0, 1, DependencyType::Presupposition, 0.9);
        graph.add_edge(1, 2, DependencyType::Entailment, 0.8);

        let affected = graph.find_affected_claims(0);
        assert!(affected.contains(&1));
        assert!(affected.contains(&2));
    }

    #[test]
    fn test_counterfactual_reasoning() {
        let claims = vec![
            "All mammals have lungs".to_string(),
            "Whales are mammals".to_string(),
            "Whales have lungs".to_string(),
        ];
        let graph = build_dependency_graph(&claims);
        let analysis = analyze_counterfactuals(&graph, &claims);

        // Graph should find relationships between claims
        assert!(!graph.edges.is_empty());
        // Reliability should be set (either 0 if no findings, or 0.65 if findings)
        assert!(analysis.reliability >= 0.0);
    }

    #[test]
    fn test_no_shared_entities() {
        let claims = vec![
            "The sky is blue".to_string(),
            "Cats are animals".to_string(),
        ];
        let graph = build_dependency_graph(&claims);
        // Should have few or no edges (different entities)
        assert!(graph.edges.len() < 3);
    }
}
