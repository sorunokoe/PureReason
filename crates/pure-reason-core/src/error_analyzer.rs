//! # Error Analysis & Pattern Detection
//!
//! Medium Win #7: Top Failure Pattern Analysis & Fixes
//!
//! This module identifies and categorizes failure modes in reasoning chains,
//! enabling targeted fixes for the highest-impact error types.
//!
//! Key error categories:
//! - Hallucination: Claims facts not supported by premises
//! - Logical fallacy: Reasoning violates logical rules
//! - Numeric error: Miscalculation or invalid arithmetic
//! - Domain mismatch: Reasoning doesn't account for domain-specific rules
//! - Contradiction: Output contradicts input or prior reasoning
//! - Over-generalization: Makes claims too broad for evidence
//! - Missing context: Ignores critical premises

use serde::{Deserialize, Serialize};

/// Type of reasoning failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorType {
    /// Unsupported claim (hallucination)
    Hallucination,
    /// Invalid logical reasoning
    LogicalFallacy,
    /// Arithmetic or numerical error
    NumericError,
    /// Violates domain-specific rules
    DomainViolation,
    /// Contradicts prior statements
    Contradiction,
    /// Claim too broad for evidence
    Overgeneralization,
    /// Missing or ignored context
    MissingContext,
    /// Ambiguous or unclear reasoning
    Ambiguity,
    /// False dichotomy or incomplete analysis
    IncompleteCasework,
    /// Other error type
    Other,
}

impl ErrorType {
    /// Human-readable name for error type.
    pub fn name(&self) -> &'static str {
        match self {
            ErrorType::Hallucination => "Hallucination",
            ErrorType::LogicalFallacy => "Logical Fallacy",
            ErrorType::NumericError => "Numeric Error",
            ErrorType::DomainViolation => "Domain Violation",
            ErrorType::Contradiction => "Contradiction",
            ErrorType::Overgeneralization => "Overgeneralization",
            ErrorType::MissingContext => "Missing Context",
            ErrorType::Ambiguity => "Ambiguity",
            ErrorType::IncompleteCasework => "Incomplete Casework",
            ErrorType::Other => "Other",
        }
    }

    /// Impact severity (1-10, where 10 is most impactful).
    pub fn severity(&self) -> usize {
        match self {
            ErrorType::Hallucination => 10, // Most dangerous
            ErrorType::Contradiction => 9,
            ErrorType::LogicalFallacy => 8,
            ErrorType::NumericError => 8,
            ErrorType::DomainViolation => 7,
            ErrorType::Overgeneralization => 6,
            ErrorType::MissingContext => 7,
            ErrorType::Ambiguity => 5,
            ErrorType::IncompleteCasework => 6,
            ErrorType::Other => 3,
        }
    }
}

/// A single error instance with context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInstance {
    /// Type of error
    pub error_type: ErrorType,
    /// Domain where error occurred
    pub domain: String,
    /// The claim that was wrong
    pub wrong_claim: String,
    /// What the correct answer should be
    pub correct_answer: String,
    /// Why this error occurred (diagnosis)
    pub diagnosis: String,
    /// How to fix this error (remediation)
    pub fix: String,
    /// Confidence in the diagnosis (0.0-1.0)
    pub confidence: f64,
}

/// Aggregated error pattern statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    /// Type of error
    pub error_type: ErrorType,
    /// How many times this error occurred
    pub frequency: usize,
    /// Percentage of all errors
    pub percentage: f64,
    /// Average severity of this error
    pub avg_severity: f64,
    /// Estimated F1 impact if fixed (0.0-0.1)
    pub f1_impact: f64,
    /// Common domains affected
    pub affected_domains: Vec<String>,
    /// Generic fix strategy
    pub generic_fix: String,
}

/// Top errors analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAnalysis {
    /// Total errors analyzed
    pub total_errors: usize,
    /// All error patterns (sorted by frequency)
    pub patterns: Vec<ErrorPattern>,
    /// Top 3 error types by frequency
    pub top_errors: Vec<ErrorType>,
    /// Top 3 errors by impact (severity * frequency)
    pub highest_impact_errors: Vec<ErrorType>,
    /// Estimated cumulative F1 improvement if all fixed
    pub total_f1_improvement: f64,
    /// Recommended fix priority (1=most impactful, easier to fix)
    pub fix_priority: Vec<(ErrorType, String)>,
}

impl ErrorAnalysis {
    /// Build analysis from error instances.
    pub fn from_errors(errors: &[ErrorInstance]) -> Self {
        if errors.is_empty() {
            return Self {
                total_errors: 0,
                patterns: Vec::new(),
                top_errors: Vec::new(),
                highest_impact_errors: Vec::new(),
                total_f1_improvement: 0.0,
                fix_priority: Vec::new(),
            };
        }

        // Count frequencies
        let mut freq_map: std::collections::HashMap<ErrorType, usize> =
            std::collections::HashMap::new();
        let mut domain_map: std::collections::HashMap<
            ErrorType,
            std::collections::HashSet<String>,
        > = std::collections::HashMap::new();

        for error in errors {
            *freq_map.entry(error.error_type).or_insert(0) += 1;
            domain_map
                .entry(error.error_type)
                .or_default()
                .insert(error.domain.clone());
        }

        let mut patterns: Vec<ErrorPattern> = freq_map
            .iter()
            .map(|(&error_type, &frequency)| {
                let percentage = (frequency as f64 / errors.len() as f64) * 100.0;
                let f1_impact = match error_type {
                    ErrorType::Hallucination => 0.080, // Fixing hallucinations helps most
                    ErrorType::NumericError => 0.050,
                    ErrorType::LogicalFallacy => 0.045,
                    ErrorType::Contradiction => 0.040,
                    ErrorType::DomainViolation => 0.035,
                    ErrorType::Overgeneralization => 0.030,
                    ErrorType::MissingContext => 0.030,
                    ErrorType::Ambiguity => 0.020,
                    ErrorType::IncompleteCasework => 0.025,
                    ErrorType::Other => 0.010,
                };

                let affected_domains = domain_map
                    .get(&error_type)
                    .map(|s| s.iter().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();

                let generic_fix = Self::generic_fix_for(error_type);

                ErrorPattern {
                    error_type,
                    frequency,
                    percentage,
                    avg_severity: error_type.severity() as f64,
                    f1_impact,
                    affected_domains,
                    generic_fix,
                }
            })
            .collect();

        // Sort by frequency descending
        patterns.sort_by(|a, b| b.frequency.cmp(&a.frequency));

        // Get top errors by frequency
        let top_errors = patterns.iter().take(3).map(|p| p.error_type).collect();

        // Get highest impact errors (severity * frequency / rarity)
        let mut impact_scores: Vec<(ErrorType, f64)> = patterns
            .iter()
            .map(|p| {
                let impact = (p.avg_severity * p.frequency as f64) / (p.percentage + 1.0);
                (p.error_type, impact)
            })
            .collect();
        impact_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let highest_impact_errors = impact_scores.iter().take(3).map(|(e, _)| *e).collect();

        let total_f1_improvement = patterns.iter().map(|p| p.f1_impact).sum();

        // Build fix priority (impact * ease)
        let mut priority: Vec<(ErrorType, String)> = patterns
            .iter()
            .map(|p| (p.error_type, p.generic_fix.clone()))
            .collect();
        // Sort by f1_impact descending (higher impact first)
        priority.sort_by(|a, b| {
            let impact_a = patterns
                .iter()
                .find(|p| p.error_type == a.0)
                .map(|p| p.f1_impact)
                .unwrap_or(0.0);
            let impact_b = patterns
                .iter()
                .find(|p| p.error_type == b.0)
                .map(|p| p.f1_impact)
                .unwrap_or(0.0);
            impact_b
                .partial_cmp(&impact_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Self {
            total_errors: errors.len(),
            patterns,
            top_errors,
            highest_impact_errors,
            total_f1_improvement,
            fix_priority: priority,
        }
    }

    /// Provide generic fix strategy for error type.
    fn generic_fix_for(error_type: ErrorType) -> String {
        match error_type {
            ErrorType::Hallucination => {
                "Add evidence requirement check: every claim must reference premises".to_string()
            }
            ErrorType::LogicalFallacy => {
                "Add formal logic validation: check for ad-hoc, appeals to authority".to_string()
            }
            ErrorType::NumericError => {
                "Add math solver validation: verify all arithmetic operations".to_string()
            }
            ErrorType::DomainViolation => {
                "Add domain-specific rule checker: validate against domain constraints".to_string()
            }
            ErrorType::Contradiction => {
                "Add consistency checker: compare output against input and prior conclusions"
                    .to_string()
            }
            ErrorType::Overgeneralization => {
                "Add evidence scope limiter: restrict claims to supported scope".to_string()
            }
            ErrorType::MissingContext => {
                "Add context analyzer: identify key premises that must not be ignored".to_string()
            }
            ErrorType::Ambiguity => {
                "Add clarification phase: request explicit definitions before reasoning".to_string()
            }
            ErrorType::IncompleteCasework => {
                "Add case enumeration: force reasoning to address all possibilities".to_string()
            }
            ErrorType::Other => "Analyze error manually to determine fix strategy".to_string(),
        }
    }
}

/// Error pattern detector for benchmark results.
pub struct ErrorDetector;

impl ErrorDetector {
    /// Detect error type from incorrect prediction.
    pub fn detect_error(
        claim: &str,
        expected: &str,
        predicted: &str,
        domain: &str,
    ) -> Option<ErrorInstance> {
        // Check for contradiction FIRST (highest priority)
        if (expected.to_lowercase().contains("no") || expected.to_lowercase().contains("false"))
            && (predicted.to_lowercase().contains("yes")
                || predicted.to_lowercase().contains("true"))
        {
            return Some(ErrorInstance {
                error_type: ErrorType::Contradiction,
                domain: domain.to_string(),
                wrong_claim: claim.to_string(),
                correct_answer: expected.to_string(),
                diagnosis: "Predicted opposite of correct answer".to_string(),
                fix: "Add negation checker and improve logical reasoning".to_string(),
                confidence: 0.9,
            });
        }

        if (expected.to_lowercase().contains("yes") || expected.to_lowercase().contains("true"))
            && (predicted.to_lowercase().contains("no")
                || predicted.to_lowercase().contains("false"))
        {
            return Some(ErrorInstance {
                error_type: ErrorType::Contradiction,
                domain: domain.to_string(),
                wrong_claim: claim.to_string(),
                correct_answer: expected.to_string(),
                diagnosis: "Predicted opposite of correct answer".to_string(),
                fix: "Add negation checker and improve logical reasoning".to_string(),
                confidence: 0.9,
            });
        }

        // Check for abstention/ambiguity
        if predicted.is_empty() || predicted == "unknown" {
            return Some(ErrorInstance {
                error_type: ErrorType::Ambiguity,
                domain: domain.to_string(),
                wrong_claim: claim.to_string(),
                correct_answer: expected.to_string(),
                diagnosis: "Model failed to produce output (abstained)".to_string(),
                fix: "Improve confidence or add fallback reasoning path".to_string(),
                confidence: 0.7,
            });
        }

        // Check for hallucination (claims not in premise)
        if !claim.to_lowercase().contains(&predicted.to_lowercase()) && expected != predicted {
            return Some(ErrorInstance {
                error_type: ErrorType::Hallucination,
                domain: domain.to_string(),
                wrong_claim: claim.to_string(),
                correct_answer: expected.to_string(),
                diagnosis: "Predicted fact not derivable from premises".to_string(),
                fix: "Add evidence requirement check and premise grounding".to_string(),
                confidence: 0.8,
            });
        }

        // Check for numeric error
        let pred_nums: Vec<f64> = predicted
            .split_whitespace()
            .filter_map(|w| w.parse().ok())
            .collect();
        let exp_nums: Vec<f64> = expected
            .split_whitespace()
            .filter_map(|w| w.parse().ok())
            .collect();

        if !pred_nums.is_empty() && !exp_nums.is_empty() && (pred_nums[0] - exp_nums[0]).abs() > 0.1
        {
            return Some(ErrorInstance {
                error_type: ErrorType::NumericError,
                domain: domain.to_string(),
                wrong_claim: claim.to_string(),
                correct_answer: expected.to_string(),
                diagnosis: "Numeric calculation or value mismatch".to_string(),
                fix: "Add math solver to verify calculations".to_string(),
                confidence: 0.85,
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_type_severity() {
        assert_eq!(ErrorType::Hallucination.severity(), 10);
        assert_eq!(ErrorType::Other.severity(), 3);
    }

    #[test]
    fn test_error_type_name() {
        assert_eq!(ErrorType::Hallucination.name(), "Hallucination");
        assert_eq!(ErrorType::NumericError.name(), "Numeric Error");
    }

    #[test]
    fn test_error_analysis_empty() {
        let analysis = ErrorAnalysis::from_errors(&[]);
        assert_eq!(analysis.total_errors, 0);
        assert!(analysis.patterns.is_empty());
    }

    #[test]
    fn test_error_analysis_single_error() {
        let error = ErrorInstance {
            error_type: ErrorType::Hallucination,
            domain: "medical".to_string(),
            wrong_claim: "Patient has condition X".to_string(),
            correct_answer: "Unknown".to_string(),
            diagnosis: "Unsupported claim".to_string(),
            fix: "Add evidence requirement".to_string(),
            confidence: 0.8,
        };

        let analysis = ErrorAnalysis::from_errors(&[error]);
        assert_eq!(analysis.total_errors, 1);
        assert_eq!(analysis.patterns.len(), 1);
        assert_eq!(analysis.patterns[0].frequency, 1);
        assert_eq!(analysis.patterns[0].percentage, 100.0);
    }

    #[test]
    fn test_error_analysis_multiple_types() {
        let errors = vec![
            ErrorInstance {
                error_type: ErrorType::Hallucination,
                domain: "medical".to_string(),
                wrong_claim: "Test".to_string(),
                correct_answer: "Answer".to_string(),
                diagnosis: "Test".to_string(),
                fix: "Test".to_string(),
                confidence: 0.8,
            },
            ErrorInstance {
                error_type: ErrorType::Hallucination,
                domain: "medical".to_string(),
                wrong_claim: "Test".to_string(),
                correct_answer: "Answer".to_string(),
                diagnosis: "Test".to_string(),
                fix: "Test".to_string(),
                confidence: 0.8,
            },
            ErrorInstance {
                error_type: ErrorType::NumericError,
                domain: "finance".to_string(),
                wrong_claim: "Test".to_string(),
                correct_answer: "Answer".to_string(),
                diagnosis: "Test".to_string(),
                fix: "Test".to_string(),
                confidence: 0.8,
            },
        ];

        let analysis = ErrorAnalysis::from_errors(&errors);
        assert_eq!(analysis.total_errors, 3);
        assert_eq!(analysis.patterns.len(), 2);
        // Hallucination should be first (more frequent)
        assert_eq!(analysis.patterns[0].error_type, ErrorType::Hallucination);
        assert_eq!(analysis.patterns[0].frequency, 2);
    }

    #[test]
    fn test_error_detector_hallucination() {
        let error = ErrorDetector::detect_error(
            "Is the sky blue?",
            "Yes",
            "Purple elephants exist",
            "general",
        );
        assert!(error.is_some());
        assert_eq!(error.unwrap().error_type, ErrorType::Hallucination);
    }

    #[test]
    fn test_error_detector_contradiction() {
        let error = ErrorDetector::detect_error("Is X true?", "No", "Yes, X is true", "logic");
        assert!(error.is_some());
        assert_eq!(error.unwrap().error_type, ErrorType::Contradiction);
    }

    #[test]
    fn test_top_errors_ranking() {
        let errors = vec![
            ErrorInstance {
                error_type: ErrorType::Hallucination,
                domain: "medical".to_string(),
                wrong_claim: "1".to_string(),
                correct_answer: "1".to_string(),
                diagnosis: "1".to_string(),
                fix: "1".to_string(),
                confidence: 0.8,
            },
            ErrorInstance {
                error_type: ErrorType::Hallucination,
                domain: "medical".to_string(),
                wrong_claim: "2".to_string(),
                correct_answer: "2".to_string(),
                diagnosis: "2".to_string(),
                fix: "2".to_string(),
                confidence: 0.8,
            },
            ErrorInstance {
                error_type: ErrorType::NumericError,
                domain: "finance".to_string(),
                wrong_claim: "3".to_string(),
                correct_answer: "3".to_string(),
                diagnosis: "3".to_string(),
                fix: "3".to_string(),
                confidence: 0.8,
            },
            ErrorInstance {
                error_type: ErrorType::LogicalFallacy,
                domain: "legal".to_string(),
                wrong_claim: "4".to_string(),
                correct_answer: "4".to_string(),
                diagnosis: "4".to_string(),
                fix: "4".to_string(),
                confidence: 0.8,
            },
        ];

        let analysis = ErrorAnalysis::from_errors(&errors);
        assert_eq!(analysis.top_errors.len(), 3);
        assert_eq!(analysis.top_errors[0], ErrorType::Hallucination);
    }

    #[test]
    fn test_f1_improvement_estimation() {
        let errors = vec![
            ErrorInstance {
                error_type: ErrorType::Hallucination,
                domain: "medical".to_string(),
                wrong_claim: "1".to_string(),
                correct_answer: "1".to_string(),
                diagnosis: "1".to_string(),
                fix: "1".to_string(),
                confidence: 0.8,
            },
            ErrorInstance {
                error_type: ErrorType::NumericError,
                domain: "finance".to_string(),
                wrong_claim: "2".to_string(),
                correct_answer: "2".to_string(),
                diagnosis: "2".to_string(),
                fix: "2".to_string(),
                confidence: 0.8,
            },
        ];

        let analysis = ErrorAnalysis::from_errors(&errors);
        // Should be sum of f1_impact for each unique error type
        assert!(analysis.total_f1_improvement > 0.1);
        assert!(analysis.total_f1_improvement < 0.15);
    }
}
