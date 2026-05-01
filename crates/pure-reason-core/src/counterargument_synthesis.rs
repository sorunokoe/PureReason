//! Counterargument Synthesis: Find, rank, and reconcile opposing arguments
//!
//! TRIZ Principle: Segmentation + Taking Out + Preliminary Action
//! Break down arguments into thesis/antithesis, identify strongest opposition,
//! and synthesize resolution following Hegelian dialectics.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// A single argument (thesis or antithesis)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argument {
    /// Statement of the argument
    pub statement: String,
    /// Perspective this represents (e.g., "medical", "economic", "ethical")
    pub perspective: String,
    /// Strength rating (0.0-1.0, reflects quality of evidence/logic)
    pub strength: f64,
    /// Key evidence or reasoning
    pub evidence: Vec<String>,
    /// Underlying assumptions
    pub assumptions: Vec<String>,
}

impl Argument {
    /// Create a new argument
    pub fn new(statement: String, perspective: String, strength: f64) -> Self {
        Self {
            statement,
            perspective,
            strength: strength.clamp(0.0, 1.0),
            evidence: vec![],
            assumptions: vec![],
        }
    }

    /// Add evidence
    pub fn with_evidence(mut self, evidence: Vec<String>) -> Self {
        self.evidence = evidence;
        self
    }

    /// Add assumptions
    pub fn with_assumptions(mut self, assumptions: Vec<String>) -> Self {
        self.assumptions = assumptions;
        self
    }

    /// Get summary
    pub fn summary(&self) -> String {
        format!(
            "[{}] {} (strength: {:.0}%)",
            self.perspective,
            self.statement,
            self.strength * 100.0
        )
    }
}

/// Dialectical contradiction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    /// Thesis (primary claim)
    pub thesis: Argument,
    /// Antithesis (opposing argument)
    pub antithesis: Argument,
    /// Common ground (shared assumptions)
    pub common_ground: Vec<String>,
    /// Points of tension (core disagreements)
    pub tension_points: Vec<String>,
}

impl Contradiction {
    /// Create a new contradiction
    pub fn new(thesis: Argument, antithesis: Argument) -> Self {
        Self {
            thesis,
            antithesis,
            common_ground: vec![],
            tension_points: vec![],
        }
    }

    /// Identify common ground
    pub fn with_common_ground(mut self, common: Vec<String>) -> Self {
        self.common_ground = common;
        self
    }

    /// Identify tension points
    pub fn with_tension_points(mut self, tensions: Vec<String>) -> Self {
        self.tension_points = tensions;
        self
    }

    /// Strength difference (how asymmetrical is the debate)
    pub fn strength_gap(&self) -> f64 {
        (self.thesis.strength - self.antithesis.strength).abs()
    }

    /// Is contradiction well-balanced?
    pub fn is_balanced(&self) -> bool {
        self.strength_gap() < 0.2
    }

    /// Dominant perspective (stronger argument)
    pub fn dominant(&self) -> &Argument {
        if self.thesis.strength >= self.antithesis.strength {
            &self.thesis
        } else {
            &self.antithesis
        }
    }

    /// Weaker perspective
    pub fn minority(&self) -> &Argument {
        if self.thesis.strength < self.antithesis.strength {
            &self.thesis
        } else {
            &self.antithesis
        }
    }
}

/// Dialectical synthesis (resolution)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synthesis {
    /// The synthesized position
    pub statement: String,
    /// Confidence in synthesis (0.0-1.0)
    pub confidence: f64,
    /// How it incorporates thesis
    pub thesis_integration: String,
    /// How it incorporates antithesis
    pub antithesis_integration: String,
    /// What assumptions must be true
    pub required_assumptions: Vec<String>,
    /// Conditions under which synthesis holds
    pub conditions: Vec<String>,
}

impl Synthesis {
    /// Create a new synthesis
    pub fn new(
        statement: String,
        confidence: f64,
        thesis_integration: String,
        antithesis_integration: String,
    ) -> Self {
        Self {
            statement,
            confidence: confidence.clamp(0.0, 1.0),
            thesis_integration,
            antithesis_integration,
            required_assumptions: vec![],
            conditions: vec![],
        }
    }

    /// Add assumptions
    pub fn with_assumptions(mut self, assumptions: Vec<String>) -> Self {
        self.required_assumptions = assumptions;
        self
    }

    /// Add conditions
    pub fn with_conditions(mut self, conditions: Vec<String>) -> Self {
        self.conditions = conditions;
        self
    }

    /// Summary
    pub fn summary(&self) -> String {
        format!(
            "Synthesis (confidence: {:.0}%): {}\n  Thesis incorporation: {}\n  Antithesis incorporation: {}",
            self.confidence * 100.0,
            self.statement,
            self.thesis_integration,
            self.antithesis_integration
        )
    }
}

/// Counterargument detector and reconciler
pub struct CounterargumentAnalyzer;

impl CounterargumentAnalyzer {
    /// Analyze a contradiction and propose synthesis
    pub fn analyze(contradiction: &Contradiction) -> Synthesis {
        let confidence = Self::calculate_synthesis_confidence(contradiction);

        let thesis_integration = format!(
            "Acknowledges {} strengths (perspective: {})",
            if contradiction.thesis.strength > 0.7 {
                "significant"
            } else {
                "important"
            },
            contradiction.thesis.perspective
        );

        let antithesis_integration = format!(
            "Acknowledges {} concerns (perspective: {})",
            if contradiction.antithesis.strength > 0.7 {
                "valid"
            } else {
                "noted"
            },
            contradiction.antithesis.perspective
        );

        Synthesis::new(
            format!(
                "Neither pure {} nor pure {}, but context-dependent",
                contradiction.thesis.perspective, contradiction.antithesis.perspective
            ),
            confidence,
            thesis_integration,
            antithesis_integration,
        )
        .with_conditions(vec![
            "Domain-specific application".to_string(),
            format!(
                "Emphasis on {} when conditions favor it",
                contradiction.dominant().perspective
            ),
            format!(
                "Consideration of {} concerns",
                contradiction.minority().perspective
            ),
        ])
    }

    /// Calculate confidence in proposed synthesis
    fn calculate_synthesis_confidence(contradiction: &Contradiction) -> f64 {
        // Higher confidence when:
        // 1. Arguments are balanced (not extreme)
        // 2. Common ground exists
        // 3. Tension points are identified (makes resolution easier)

        let balance_factor = 1.0 - contradiction.strength_gap();
        let ground_factor = if contradiction.common_ground.is_empty() {
            0.5
        } else {
            0.8
        };
        let tension_factor = if contradiction.tension_points.is_empty() {
            0.6
        } else {
            0.9
        };

        (balance_factor * 0.3 + ground_factor * 0.3 + tension_factor * 0.4).clamp(0.5, 0.95)
    }

    /// Rank arguments by strength
    pub fn rank_arguments(arguments: &[Argument]) -> Vec<(usize, &Argument)> {
        let mut ranked: Vec<_> = arguments.iter().enumerate().collect();
        ranked.sort_by(|a, b| {
            b.1.strength
                .partial_cmp(&a.1.strength)
                .unwrap_or(Ordering::Equal)
        });
        ranked.into_iter().collect()
    }

    /// Find best thesis/antithesis pair from arguments
    pub fn find_best_contradiction(arguments: &[Argument]) -> Option<(usize, usize)> {
        if arguments.len() < 2 {
            return None;
        }

        let ranked = Self::rank_arguments(arguments);
        if ranked.len() >= 2 {
            Some((ranked[0].0, ranked[1].0))
        } else {
            None
        }
    }

    /// Explain the contradiction
    pub fn explain(contradiction: &Contradiction) -> String {
        let mut explanation = String::new();
        explanation.push_str("## Dialectical Analysis\n\n");

        explanation.push_str(&format!(
            "**Thesis**: {}\n  - Perspective: {}\n  - Strength: {:.0}%\n\n",
            contradiction.thesis.statement,
            contradiction.thesis.perspective,
            contradiction.thesis.strength * 100.0
        ));

        explanation.push_str(&format!(
            "**Antithesis**: {}\n  - Perspective: {}\n  - Strength: {:.0}%\n\n",
            contradiction.antithesis.statement,
            contradiction.antithesis.perspective,
            contradiction.antithesis.strength * 100.0
        ));

        explanation.push_str(&format!(
            "**Strength Gap**: {:.0}% ({})\n\n",
            contradiction.strength_gap() * 100.0,
            if contradiction.is_balanced() {
                "balanced debate"
            } else {
                "asymmetrical debate"
            }
        ));

        if !contradiction.common_ground.is_empty() {
            explanation.push_str("**Common Ground**:\n");
            for item in &contradiction.common_ground {
                explanation.push_str(&format!("  - {}\n", item));
            }
            explanation.push('\n');
        }

        if !contradiction.tension_points.is_empty() {
            explanation.push_str("**Core Tensions**:\n");
            for item in &contradiction.tension_points {
                explanation.push_str(&format!("  - {}\n", item));
            }
            explanation.push('\n');
        }

        explanation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argument_creation() {
        let arg = Argument::new(
            "Universal basic income reduces poverty".to_string(),
            "economic".to_string(),
            0.85,
        );
        assert_eq!(arg.strength, 0.85);
        assert_eq!(arg.perspective, "economic");
    }

    #[test]
    fn test_argument_with_evidence() {
        let arg = Argument::new("UBI works".to_string(), "economic".to_string(), 0.80)
            .with_evidence(vec!["Pilot study".to_string(), "Survey data".to_string()]);
        assert_eq!(arg.evidence.len(), 2);
    }

    #[test]
    fn test_contradiction_creation() {
        let thesis = Argument::new(
            "UBI reduces poverty".to_string(),
            "social".to_string(),
            0.85,
        );
        let antithesis = Argument::new(
            "UBI causes inflation".to_string(),
            "economic".to_string(),
            0.75,
        );
        let contra = Contradiction::new(thesis, antithesis);
        assert!((contra.strength_gap() - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_contradiction_balance() {
        let thesis = Argument::new("A".to_string(), "p1".to_string(), 0.85);
        let antithesis = Argument::new("B".to_string(), "p2".to_string(), 0.80);
        let contra = Contradiction::new(thesis, antithesis);
        assert!(contra.is_balanced());
    }

    #[test]
    fn test_contradiction_imbalance() {
        let thesis = Argument::new("A".to_string(), "p1".to_string(), 0.95);
        let antithesis = Argument::new("B".to_string(), "p2".to_string(), 0.50);
        let contra = Contradiction::new(thesis, antithesis);
        assert!(!contra.is_balanced());
    }

    #[test]
    fn test_dominant_argument() {
        let thesis = Argument::new("A".to_string(), "p1".to_string(), 0.90);
        let antithesis = Argument::new("B".to_string(), "p2".to_string(), 0.70);
        let contra = Contradiction::new(thesis, antithesis);
        assert_eq!(contra.dominant().strength, 0.90);
        assert_eq!(contra.minority().strength, 0.70);
    }

    #[test]
    fn test_synthesis_creation() {
        let synthesis = Synthesis::new(
            "Nuanced position".to_string(),
            0.80,
            "Incorporates A".to_string(),
            "Incorporates B".to_string(),
        );
        assert_eq!(synthesis.confidence, 0.80);
    }

    #[test]
    fn test_rank_arguments() {
        let args = vec![
            Argument::new("A".to_string(), "p1".to_string(), 0.70),
            Argument::new("B".to_string(), "p2".to_string(), 0.90),
            Argument::new("C".to_string(), "p3".to_string(), 0.80),
        ];
        let ranked = CounterargumentAnalyzer::rank_arguments(&args);
        assert_eq!(ranked[0].1.strength, 0.90);
        assert_eq!(ranked[1].1.strength, 0.80);
        assert_eq!(ranked[2].1.strength, 0.70);
    }

    #[test]
    fn test_find_best_contradiction() {
        let args = vec![
            Argument::new("Strong".to_string(), "p1".to_string(), 0.90),
            Argument::new("Medium".to_string(), "p2".to_string(), 0.75),
            Argument::new("Weak".to_string(), "p3".to_string(), 0.50),
        ];
        let pair = CounterargumentAnalyzer::find_best_contradiction(&args).unwrap();
        assert_eq!(pair, (0, 1)); // Strongest vs second strongest
    }

    #[test]
    fn test_analyze_contradiction() {
        let thesis = Argument::new("UBI works".to_string(), "social".to_string(), 0.85);
        let antithesis = Argument::new("UBI fails".to_string(), "economic".to_string(), 0.75);
        let contra = Contradiction::new(thesis, antithesis)
            .with_common_ground(vec!["Need to help poor".to_string()])
            .with_tension_points(vec!["Cost vs effectiveness".to_string()]);
        let synthesis = CounterargumentAnalyzer::analyze(&contra);
        assert!(synthesis.confidence > 0.5);
    }

    #[test]
    fn test_explain_contradiction() {
        let thesis = Argument::new("A".to_string(), "econ".to_string(), 0.85);
        let antithesis = Argument::new("B".to_string(), "social".to_string(), 0.70);
        let contra = Contradiction::new(thesis, antithesis);
        let explanation = CounterargumentAnalyzer::explain(&contra);
        assert!(explanation.contains("Thesis"));
        assert!(explanation.contains("Antithesis"));
        assert!(explanation.contains("Strength Gap"));
    }

    #[test]
    fn test_argument_summary() {
        let arg = Argument::new("Test statement".to_string(), "legal".to_string(), 0.82);
        let summary = arg.summary();
        assert!(summary.contains("legal"));
        assert!(summary.contains("82%"));
    }

    #[test]
    fn test_synthesis_summary() {
        let synthesis = Synthesis::new(
            "Test".to_string(),
            0.75,
            "Inc A".to_string(),
            "Inc B".to_string(),
        );
        let summary = synthesis.summary();
        assert!(summary.contains("75%"));
        assert!(summary.contains("Inc A"));
    }

    #[test]
    fn test_empty_arguments_find_contradiction() {
        let args: Vec<Argument> = vec![];
        let result = CounterargumentAnalyzer::find_best_contradiction(&args);
        assert!(result.is_none());
    }
}
