//! Assumption Validation: Extract and validate implicit premises
//!
//! TRIZ Principle: Taking Out + Transition to Micro-Level
//! Identify hidden assumptions in logical arguments, grade their validity,
//! and detect when assumptions invalidate conclusions.

use serde::{Deserialize, Serialize};

/// Type of logical relationship
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogicalForm {
    /// If A then B (conditional)
    Conditional,
    /// A and B (conjunction)
    Conjunction,
    /// A or B (disjunction)
    Disjunction,
    /// Not A (negation)
    Negation,
    /// All A are B (universal)
    Universal,
}

impl std::fmt::Display for LogicalForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Conditional => write!(f, "Conditional (if-then)"),
            Self::Conjunction => write!(f, "Conjunction (and)"),
            Self::Disjunction => write!(f, "Disjunction (or)"),
            Self::Negation => write!(f, "Negation (not)"),
            Self::Universal => write!(f, "Universal (all)"),
        }
    }
}

/// Type of assumption
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssumptionType {
    /// Assumption is required for conclusion to hold
    Necessary,
    /// Assumption is sufficient but not required
    Sufficient,
    /// Assumption provides context but not strictly required
    Contextual,
    /// Assumption contradicts conclusion
    Contradictory,
}

impl std::fmt::Display for AssumptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Necessary => write!(f, "Necessary"),
            Self::Sufficient => write!(f, "Sufficient"),
            Self::Contextual => write!(f, "Contextual"),
            Self::Contradictory => write!(f, "Contradictory"),
        }
    }
}

/// A single assumption (implicit premise)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assumption {
    /// The implicit premise
    pub statement: String,
    /// Type of assumption
    pub assumption_type: AssumptionType,
    /// How widely accepted this assumption is (0.0-1.0)
    pub acceptance: f64,
    /// Evidence for or against this assumption
    pub evidence: Vec<String>,
    /// Whether this assumption is valid (can be proven/accepted)
    pub is_valid: bool,
}

impl Assumption {
    /// Create a new assumption
    pub fn new(statement: String, assumption_type: AssumptionType, acceptance: f64) -> Self {
        Self {
            statement,
            assumption_type,
            acceptance: acceptance.clamp(0.0, 1.0),
            evidence: vec![],
            is_valid: acceptance > 0.5,
        }
    }

    /// Add evidence
    pub fn with_evidence(mut self, evidence: Vec<String>) -> Self {
        self.evidence = evidence;
        self
    }

    /// Mark validity
    pub fn with_validity(mut self, is_valid: bool) -> Self {
        self.is_valid = is_valid;
        self
    }

    /// Summary
    pub fn summary(&self) -> String {
        format!(
            "[{}] {} (acceptance: {:.0}%, valid: {})",
            self.assumption_type,
            self.statement,
            self.acceptance * 100.0,
            self.is_valid
        )
    }
}

/// An argument with extracted logical structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalArgument {
    /// The full claim or argument
    pub claim: String,
    /// Logical form
    pub logical_form: LogicalForm,
    /// Explicit premises
    pub explicit_premises: Vec<String>,
    /// Conclusion
    pub conclusion: String,
    /// Extracted assumptions (implicit premises)
    pub assumptions: Vec<Assumption>,
    /// Overall argument soundness (0.0-1.0)
    pub soundness: f64,
}

impl LogicalArgument {
    /// Create a new logical argument
    pub fn new(
        claim: String,
        logical_form: LogicalForm,
        explicit_premises: Vec<String>,
        conclusion: String,
    ) -> Self {
        Self {
            claim,
            logical_form,
            explicit_premises,
            conclusion,
            assumptions: vec![],
            soundness: 0.5,
        }
    }

    /// Add assumptions
    pub fn with_assumptions(mut self, assumptions: Vec<Assumption>) -> Self {
        self.soundness = Self::calculate_soundness(&self.explicit_premises, &assumptions);
        self.assumptions = assumptions;
        self
    }

    /// Calculate soundness based on premises and assumptions
    fn calculate_soundness(premises: &[String], assumptions: &[Assumption]) -> f64 {
        if premises.is_empty() {
            return 0.3;
        }

        let premise_factor = 0.7;
        let assumption_factor = if assumptions.is_empty() {
            0.8
        } else {
            let valid_count = assumptions.iter().filter(|a| a.is_valid).count();
            let acceptance_avg =
                assumptions.iter().map(|a| a.acceptance).sum::<f64>() / assumptions.len() as f64;
            valid_count as f64 / assumptions.len() as f64 * 0.5 + acceptance_avg * 0.5
        };

        (premise_factor * 0.5 + assumption_factor * 0.5).clamp(0.0, 1.0)
    }

    /// Check if argument is logically sound
    pub fn is_sound(&self) -> bool {
        self.soundness > 0.7
            && self
                .assumptions
                .iter()
                .filter(|a| a.assumption_type == AssumptionType::Contradictory)
                .count()
                == 0
    }

    /// Identify critical assumptions (necessary and not widely accepted)
    pub fn critical_assumptions(&self) -> Vec<&Assumption> {
        self.assumptions
            .iter()
            .filter(|a| a.assumption_type == AssumptionType::Necessary && a.acceptance < 0.7)
            .collect()
    }

    /// Summary
    pub fn summary(&self) -> String {
        format!(
            "Argument soundness: {:.0}% ({} assumptions, {} critical)",
            self.soundness * 100.0,
            self.assumptions.len(),
            self.critical_assumptions().len()
        )
    }
}

/// Assumption analyzer
pub struct AssumptionValidator;

impl AssumptionValidator {
    /// Extract assumptions from an argument
    pub fn extract(argument: &LogicalArgument) -> LogicalArgument {
        let mut extracted = argument.clone();

        let assumptions = match argument.logical_form {
            LogicalForm::Conditional => Self::extract_conditional_assumptions(&argument.conclusion),
            LogicalForm::Conjunction => {
                Self::extract_conjunction_assumptions(&argument.explicit_premises)
            }
            LogicalForm::Universal => Self::extract_universal_assumptions(&argument.conclusion),
            _ => vec![],
        };

        extracted.soundness =
            LogicalArgument::calculate_soundness(&argument.explicit_premises, &assumptions);
        extracted.assumptions = assumptions;
        extracted
    }

    /// Extract assumptions from conditional statements
    fn extract_conditional_assumptions(_conclusion: &str) -> Vec<Assumption> {
        vec![
            Assumption::new(
                "Causal mechanism is valid".to_string(),
                AssumptionType::Necessary,
                0.6,
            )
            .with_validity(true),
            Assumption::new(
                "Confounding variables don't invalidate relationship".to_string(),
                AssumptionType::Necessary,
                0.5,
            )
            .with_validity(false),
            Assumption::new(
                "Temporal ordering is correct".to_string(),
                AssumptionType::Necessary,
                0.7,
            )
            .with_validity(true),
        ]
    }

    /// Extract assumptions from conjunctions
    fn extract_conjunction_assumptions(premises: &[String]) -> Vec<Assumption> {
        let mut assumptions = vec![];

        if premises.len() > 1 {
            assumptions.push(
                Assumption::new(
                    "All premises are compatible".to_string(),
                    AssumptionType::Necessary,
                    0.75,
                )
                .with_validity(true),
            );

            assumptions.push(
                Assumption::new(
                    "No hidden contradictions exist".to_string(),
                    AssumptionType::Necessary,
                    0.65,
                )
                .with_validity(true),
            );
        }

        assumptions
    }

    /// Extract assumptions from universal statements
    fn extract_universal_assumptions(_conclusion: &str) -> Vec<Assumption> {
        vec![
            Assumption::new(
                "Sample is representative of population".to_string(),
                AssumptionType::Necessary,
                0.7,
            )
            .with_validity(true),
            Assumption::new(
                "No selection bias exists".to_string(),
                AssumptionType::Necessary,
                0.55,
            )
            .with_validity(false),
            Assumption::new(
                "Context doesn't invalidate universality".to_string(),
                AssumptionType::Contextual,
                0.60,
            )
            .with_validity(true),
        ]
    }

    /// Validate all assumptions in an argument
    pub fn validate(argument: &LogicalArgument) -> LogicalArgument {
        let mut validated = argument.clone();

        for assumption in &mut validated.assumptions {
            // Mark as valid if widely accepted or has evidence
            if assumption.acceptance > 0.7 || !assumption.evidence.is_empty() {
                assumption.is_valid = true;
            }

            // Mark invalid if contradictory
            if assumption.assumption_type == AssumptionType::Contradictory {
                assumption.is_valid = false;
            }
        }

        validated.soundness = LogicalArgument::calculate_soundness(
            &argument.explicit_premises,
            &validated.assumptions,
        );

        validated
    }

    /// Find invalidating assumptions (which would make conclusion false)
    pub fn find_invalidating(argument: &LogicalArgument) -> Vec<&Assumption> {
        argument
            .assumptions
            .iter()
            .filter(|a| a.assumption_type == AssumptionType::Necessary && !a.is_valid)
            .collect()
    }

    /// Generate explanation of assumptions
    pub fn explain(argument: &LogicalArgument) -> String {
        let mut explanation = String::new();

        explanation.push_str("## Assumption Analysis\n\n");
        explanation.push_str(&format!(
            "**Claim**: {}\n**Logical Form**: {}\n",
            argument.claim, argument.logical_form
        ));

        if !argument.explicit_premises.is_empty() {
            explanation.push_str("\n**Explicit Premises**:\n");
            for (i, premise) in argument.explicit_premises.iter().enumerate() {
                explanation.push_str(&format!("  {}. {}\n", i + 1, premise));
            }
        }

        explanation.push_str(&format!("\n**Conclusion**: {}\n", argument.conclusion));

        if !argument.assumptions.is_empty() {
            explanation.push_str("\n**Implicit Assumptions**:\n");
            for assumption in &argument.assumptions {
                explanation.push_str(&format!(
                    "  - {} ({}): {}\n    Type: {}, Acceptance: {:.0}%, Valid: {}\n",
                    assumption.statement,
                    if assumption.is_valid { "✓" } else { "✗" },
                    if !assumption.evidence.is_empty() {
                        format!("Evidence: {}", assumption.evidence.join(", "))
                    } else {
                        "No evidence provided".to_string()
                    },
                    assumption.assumption_type,
                    assumption.acceptance * 100.0,
                    assumption.is_valid
                ));
            }
        }

        let critical = argument.critical_assumptions();
        if !critical.is_empty() {
            explanation.push_str("\n**⚠️ Critical Assumptions** (may invalidate conclusion):\n");
            for assumption in critical {
                explanation.push_str(&format!(
                    "  - {} (acceptance: {:.0}%)\n",
                    assumption.statement,
                    assumption.acceptance * 100.0
                ));
            }
        }

        explanation.push_str(&format!(
            "\n**Overall Soundness**: {:.0}%\n",
            argument.soundness * 100.0
        ));

        if argument.is_sound() {
            explanation.push_str("✓ Argument appears logically sound\n");
        } else {
            explanation.push_str("✗ Argument has logical weaknesses\n");
        }

        explanation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assumption_creation() {
        let ass = Assumption::new("Test".to_string(), AssumptionType::Necessary, 0.75);
        assert_eq!(ass.assumption_type, AssumptionType::Necessary);
        assert_eq!(ass.acceptance, 0.75);
    }

    #[test]
    fn test_assumption_with_evidence() {
        let ass = Assumption::new("Test".to_string(), AssumptionType::Necessary, 0.75)
            .with_evidence(vec!["Evidence 1".to_string()]);
        assert_eq!(ass.evidence.len(), 1);
    }

    #[test]
    fn test_logical_argument_creation() {
        let arg = LogicalArgument::new(
            "If A then B".to_string(),
            LogicalForm::Conditional,
            vec!["A is true".to_string()],
            "Therefore B".to_string(),
        );
        assert_eq!(arg.logical_form, LogicalForm::Conditional);
    }

    #[test]
    fn test_assumption_types_display() {
        assert_eq!(format!("{}", AssumptionType::Necessary), "Necessary");
        assert_eq!(format!("{}", AssumptionType::Sufficient), "Sufficient");
    }

    #[test]
    fn test_logical_form_display() {
        assert_eq!(
            format!("{}", LogicalForm::Conditional),
            "Conditional (if-then)"
        );
    }

    #[test]
    fn test_extract_conditional_assumptions() {
        let assumptions = AssumptionValidator::extract_conditional_assumptions("If X then Y");
        assert!(!assumptions.is_empty());
        assert!(assumptions
            .iter()
            .any(|a| a.assumption_type == AssumptionType::Necessary));
    }

    #[test]
    fn test_extract_universal_assumptions() {
        let assumptions = AssumptionValidator::extract_universal_assumptions("All X are Y");
        assert!(!assumptions.is_empty());
    }

    #[test]
    fn test_validate_assumptions() {
        let arg = LogicalArgument::new(
            "Test".to_string(),
            LogicalForm::Conditional,
            vec!["Premise".to_string()],
            "Conclusion".to_string(),
        )
        .with_assumptions(vec![
            Assumption::new("Ass1".to_string(), AssumptionType::Necessary, 0.85),
            Assumption::new("Ass2".to_string(), AssumptionType::Necessary, 0.40),
        ]);

        let validated = AssumptionValidator::validate(&arg);
        assert!(validated.assumptions[0].is_valid);
    }

    #[test]
    fn test_critical_assumptions() {
        let arg = LogicalArgument::new(
            "Test".to_string(),
            LogicalForm::Conditional,
            vec!["Premise".to_string()],
            "Conclusion".to_string(),
        )
        .with_assumptions(vec![
            Assumption::new("Critical".to_string(), AssumptionType::Necessary, 0.4),
            Assumption::new("Safe".to_string(), AssumptionType::Necessary, 0.9),
        ]);

        let critical = arg.critical_assumptions();
        assert_eq!(critical.len(), 1);
    }

    #[test]
    fn test_find_invalidating() {
        let arg = LogicalArgument::new(
            "Test".to_string(),
            LogicalForm::Conditional,
            vec!["P".to_string()],
            "C".to_string(),
        )
        .with_assumptions(vec![Assumption::new(
            "A".to_string(),
            AssumptionType::Necessary,
            0.3,
        )
        .with_validity(false)]);

        let invalidating = AssumptionValidator::find_invalidating(&arg);
        assert!(!invalidating.is_empty());
    }

    #[test]
    fn test_is_sound_true() {
        let arg = LogicalArgument::new(
            "Test".to_string(),
            LogicalForm::Conditional,
            vec!["P1".to_string(), "P2".to_string()],
            "C".to_string(),
        )
        .with_assumptions(vec![Assumption::new(
            "A1".to_string(),
            AssumptionType::Sufficient,
            0.85,
        )
        .with_validity(true)]);

        assert!(arg.is_sound());
    }

    #[test]
    fn test_explain_argument() {
        let arg = LogicalArgument::new(
            "Test".to_string(),
            LogicalForm::Conditional,
            vec!["If A then B".to_string()],
            "B".to_string(),
        );

        let explanation = AssumptionValidator::explain(&arg);
        assert!(explanation.contains("Assumption Analysis"));
    }

    #[test]
    fn test_assumption_summary() {
        let ass = Assumption::new("Test".to_string(), AssumptionType::Necessary, 0.75);
        let summary = ass.summary();
        assert!(summary.contains("Necessary"));
        assert!(summary.contains("75%"));
    }

    #[test]
    fn test_logical_argument_summary() {
        let arg = LogicalArgument::new(
            "Test".to_string(),
            LogicalForm::Conditional,
            vec!["P".to_string()],
            "C".to_string(),
        );
        let summary = arg.summary();
        assert!(summary.contains("soundness"));
    }

    #[test]
    fn test_soundness_calculation() {
        let arg = LogicalArgument::new(
            "Test".to_string(),
            LogicalForm::Conditional,
            vec!["P".to_string()],
            "C".to_string(),
        )
        .with_assumptions(vec![Assumption::new(
            "A".to_string(),
            AssumptionType::Necessary,
            0.85,
        )
        .with_validity(true)]);

        assert!(arg.soundness > 0.0 && arg.soundness <= 1.0);
    }
}
