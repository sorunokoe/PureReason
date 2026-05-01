// Neurosymbolic Verification Layer - Phase 3.5.1
// Combines heuristic reasoning with symbolic constraint checking
// Detects hallucinations and validates against domain-specific rules

use crate::domain_config::Domain;

/// Symbolic constraint that must be satisfied for valid reasoning
#[derive(Debug, Clone)]
pub struct Constraint {
    pub name: String,
    pub description: String,
    pub domain: Domain,
    pub severity: ConstraintSeverity,
    pub check_fn: fn(&str) -> bool,
}

/// Severity level determines confidence penalty if violated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintSeverity {
    Critical, // Violation = confidence penalty -0.25
    High,     // Violation = confidence penalty -0.15
    Medium,   // Violation = confidence penalty -0.08
    Low,      // Violation = confidence penalty -0.03
}

impl ConstraintSeverity {
    pub fn penalty(&self) -> f64 {
        match self {
            ConstraintSeverity::Critical => 0.25,
            ConstraintSeverity::High => 0.15,
            ConstraintSeverity::Medium => 0.08,
            ConstraintSeverity::Low => 0.03,
        }
    }
}

/// Result of symbolic verification
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub verified: bool,
    pub violations: Vec<ConstraintViolation>,
    pub confidence_penalty: f64,
}

/// Individual constraint violation
#[derive(Debug, Clone)]
pub struct ConstraintViolation {
    pub constraint_name: String,
    pub severity: ConstraintSeverity,
    pub description: String,
}

/// Symbolic verifier for domain-specific reasoning validation
pub struct SymbolicVerifier {
    _domain: Domain, // Stored for future domain-specific calibration features
    constraints: Vec<Constraint>,
}

impl SymbolicVerifier {
    /// Create verifier for a specific domain
    pub fn for_domain(domain: Domain) -> Self {
        let constraints = Self::get_domain_constraints(domain);
        SymbolicVerifier {
            _domain: domain,
            constraints,
        }
    }

    /// Get domain-specific constraints
    fn get_domain_constraints(domain: Domain) -> Vec<Constraint> {
        match domain {
            Domain::Medical => Self::medical_constraints(),
            Domain::Legal => Self::legal_constraints(),
            Domain::Finance => Self::finance_constraints(),
            Domain::Science => Self::science_constraints(),
            Domain::Code => Self::code_constraints(),
            Domain::General => vec![], // No constraints for general domain
        }
    }

    /// Verify reasoning output against domain constraints
    pub fn verify_reasoning(&self, claim: &str) -> VerificationResult {
        let mut violations = Vec::new();

        for constraint in &self.constraints {
            if !(constraint.check_fn)(claim) {
                violations.push(ConstraintViolation {
                    constraint_name: constraint.name.clone(),
                    severity: constraint.severity,
                    description: constraint.description.clone(),
                });
            }
        }

        // Compute confidence penalty from violations
        let confidence_penalty: f64 = violations.iter().map(|v| v.severity.penalty()).sum();
        let confidence_penalty = confidence_penalty.min(0.5); // Cap at -0.50

        VerificationResult {
            verified: violations.is_empty(),
            violations,
            confidence_penalty,
        }
    }

    // Domain-specific constraint definitions

    fn medical_constraints() -> Vec<Constraint> {
        vec![
            Constraint {
                name: "has_dosage_specification".to_string(),
                description: "Medical recommendations must specify dosage or equivalent"
                    .to_string(),
                domain: Domain::Medical,
                severity: ConstraintSeverity::High,
                check_fn: |claim| {
                    let lower = claim.to_lowercase();
                    lower.contains("mg")
                        || lower.contains("ml")
                        || lower.contains("unit")
                        || lower.contains("dose")
                        || lower.contains("quantity")
                },
            },
            Constraint {
                name: "mentions_source_or_standard".to_string(),
                description:
                    "Medical claims should reference FDA, peer-reviewed, or clinical source"
                        .to_string(),
                domain: Domain::Medical,
                severity: ConstraintSeverity::High,
                check_fn: |claim| {
                    let lower = claim.to_lowercase();
                    lower.contains("fda")
                        || lower.contains("peer")
                        || lower.contains("clinical")
                        || lower.contains("standard")
                        || lower.contains("guideline")
                        || lower.contains("protocol")
                        || lower.contains("study")
                },
            },
            Constraint {
                name: "avoids_absolute_claims".to_string(),
                description: "Avoid absolute certainty language ('always', 'never', 'guaranteed')"
                    .to_string(),
                domain: Domain::Medical,
                severity: ConstraintSeverity::Medium,
                check_fn: |claim| {
                    let lower = claim.to_lowercase();
                    !lower.contains("always works")
                        && !lower.contains("never fails")
                        && !lower.contains("guaranteed cure")
                },
            },
        ]
    }

    fn legal_constraints() -> Vec<Constraint> {
        vec![
            Constraint {
                name: "references_jurisdiction".to_string(),
                description: "Legal claims should reference jurisdiction or applicable law"
                    .to_string(),
                domain: Domain::Legal,
                severity: ConstraintSeverity::High,
                check_fn: |claim| {
                    let lower = claim.to_lowercase();
                    lower.contains("court")
                        || lower.contains("state")
                        || lower.contains("federal")
                        || lower.contains("law")
                        || lower.contains("statute")
                        || lower.contains("regulation")
                        || lower.contains("jurisdiction")
                        || lower.contains("district")
                },
            },
            Constraint {
                name: "cites_sources".to_string(),
                description:
                    "Legal conclusions should cite relevant cases, statutes, or regulations"
                        .to_string(),
                domain: Domain::Legal,
                severity: ConstraintSeverity::High,
                check_fn: |claim| {
                    let lower = claim.to_lowercase();
                    lower.contains("case")
                        || lower.contains("statute")
                        || lower.contains("code")
                        || lower.contains("act")
                        || lower.contains("article")
                        || lower.contains("section")
                        || lower.contains("§")
                        || lower.contains("u.s.c.")
                },
            },
            Constraint {
                name: "logical_consistency".to_string(),
                description: "Avoid logical contradictions".to_string(),
                domain: Domain::Legal,
                severity: ConstraintSeverity::Critical,
                check_fn: |claim| {
                    let lower = claim.to_lowercase();
                    // Check for obvious contradictions
                    let has_both_and_neither = lower.contains("and") && lower.contains("neither");
                    let has_required_and_prohibited =
                        lower.contains("required") && lower.contains("prohibited");
                    !has_both_and_neither && !has_required_and_prohibited
                },
            },
        ]
    }

    fn finance_constraints() -> Vec<Constraint> {
        vec![
            Constraint {
                name: "numeric_values_present".to_string(),
                description: "Financial recommendations should include specific numbers"
                    .to_string(),
                domain: Domain::Finance,
                severity: ConstraintSeverity::High,
                check_fn: |claim| {
                    // Check for numbers or percentages
                    claim.chars().any(|c| c.is_numeric()) || claim.contains("%")
                },
            },
            Constraint {
                name: "references_standard_formula".to_string(),
                description:
                    "Financial calculations should reference standard formulas (DCF, CAPM, NPV)"
                        .to_string(),
                domain: Domain::Finance,
                severity: ConstraintSeverity::Medium,
                check_fn: |claim| {
                    let lower = claim.to_lowercase();
                    lower.contains("dcf")
                        || lower.contains("capm")
                        || lower.contains("npv")
                        || lower.contains("discount")
                        || lower.contains("rate of return")
                        || lower.contains("formula")
                        || lower.contains("model")
                },
            },
            Constraint {
                name: "states_assumptions".to_string(),
                description: "Financial forecasts should explicitly state assumptions".to_string(),
                domain: Domain::Finance,
                severity: ConstraintSeverity::Medium,
                check_fn: |claim| {
                    let lower = claim.to_lowercase();
                    lower.contains("assume")
                        || lower.contains("based on")
                        || lower.contains("given")
                        || lower.contains("if")
                },
            },
        ]
    }

    fn science_constraints() -> Vec<Constraint> {
        vec![
            Constraint {
                name: "supports_with_evidence".to_string(),
                description: "Scientific claims should reference studies, data, or mechanisms"
                    .to_string(),
                domain: Domain::Science,
                severity: ConstraintSeverity::High,
                check_fn: |claim| {
                    let lower = claim.to_lowercase();
                    lower.contains("study")
                        || lower.contains("research")
                        || lower.contains("data")
                        || lower.contains("experiment")
                        || lower.contains("evidence")
                        || lower.contains("mechanism")
                        || lower.contains("theory")
                },
            },
            Constraint {
                name: "acknowledges_limitations".to_string(),
                description: "Should acknowledge limitations or uncertainties in scientific claims"
                    .to_string(),
                domain: Domain::Science,
                severity: ConstraintSeverity::Medium,
                check_fn: |claim| {
                    let lower = claim.to_lowercase();
                    lower.contains("may")
                        || lower.contains("could")
                        || lower.contains("suggest")
                        || lower.contains("indicate")
                        || lower.contains("appears")
                        || lower.contains("limited")
                        || lower.contains("unclear")
                },
            },
        ]
    }

    fn code_constraints() -> Vec<Constraint> {
        vec![
            Constraint {
                name: "syntax_valid".to_string(),
                description: "Code recommendations should have valid syntax".to_string(),
                domain: Domain::Code,
                severity: ConstraintSeverity::Critical,
                check_fn: |claim| {
                    // Very basic: check for matching braces/parens
                    let open_braces = claim.matches('{').count();
                    let close_braces = claim.matches('}').count();
                    let open_parens = claim.matches('(').count();
                    let close_parens = claim.matches(')').count();
                    open_braces == close_braces && open_parens == close_parens
                },
            },
            Constraint {
                name: "references_language_or_library".to_string(),
                description: "Code recommendations should reference the language or library"
                    .to_string(),
                domain: Domain::Code,
                severity: ConstraintSeverity::Medium,
                check_fn: |claim| {
                    let lower = claim.to_lowercase();
                    lower.contains("python")
                        || lower.contains("rust")
                        || lower.contains("javascript")
                        || lower.contains("java")
                        || lower.contains("function")
                        || lower.contains("class")
                        || lower.contains("method")
                        || lower.contains("library")
                        || lower.contains("import")
                },
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_medical_dosage_constraint() {
        let verifier = SymbolicVerifier::for_domain(Domain::Medical);

        // GIVEN: Claim without dosage
        let bad_claim = "Drug X is safe for patients.";
        let result = verifier.verify_reasoning(bad_claim);

        // THEN: Constraint violation detected
        assert!(!result.verified);
        assert!(result
            .violations
            .iter()
            .any(|v| v.constraint_name == "has_dosage_specification"));
        assert!(result.confidence_penalty > 0.1);
    }

    #[test]
    fn test_medical_with_proper_specification() {
        let verifier = SymbolicVerifier::for_domain(Domain::Medical);

        // GIVEN: Claim with proper dosage and source
        let good_claim = "Drug X at 2mg/kg/day following FDA guidelines is safe.";
        let result = verifier.verify_reasoning(good_claim);

        // THEN: Fewer violations
        assert!(result
            .violations
            .iter()
            .all(|v| v.constraint_name != "has_dosage_specification"));
    }

    #[test]
    fn test_legal_logic_contradiction_detection() {
        let verifier = SymbolicVerifier::for_domain(Domain::Legal);

        // GIVEN: Logically contradictory claim
        let contradictory = "The defendant is required and prohibited from attending.";
        let result = verifier.verify_reasoning(contradictory);

        // THEN: Critical violation detected
        let critical_violations: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.severity == ConstraintSeverity::Critical)
            .collect();
        assert!(!critical_violations.is_empty());
    }

    #[test]
    fn test_finance_numeric_requirement() {
        let verifier = SymbolicVerifier::for_domain(Domain::Finance);

        // GIVEN: Financial claim with numbers
        let numeric_claim = "Investment at 7.5% annual return over 10 years yields $21,500.";
        let result = verifier.verify_reasoning(numeric_claim);

        // THEN: Passes numeric validation
        assert!(!result
            .violations
            .iter()
            .any(|v| v.constraint_name == "numeric_values_present"));
    }

    #[test]
    fn test_code_brace_matching() {
        let verifier = SymbolicVerifier::for_domain(Domain::Code);

        // GIVEN: Code with unmatched braces
        let unmatched = "if (x > 0) { return x; ";
        let result = verifier.verify_reasoning(unmatched);

        // THEN: Syntax error detected
        assert!(result
            .violations
            .iter()
            .any(|v| v.constraint_name == "syntax_valid"));
    }
}
