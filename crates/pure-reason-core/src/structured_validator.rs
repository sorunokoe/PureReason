//! # Structured Decision Validator (S-IV-9)
//!
//! Validates AI-generated structured outputs (JSON decisions) against both
//! schema constraints AND Kantian epistemic constraints.
//!
//! Detects:
//! 1. Internal field contradictions (e.g., conservative risk + 45% crypto allocation)
//! 2. Epistemic overreach in string field values
//! 3. Domain constraint violations in structured content

use crate::domain::{builtin_profile, DomainProfile};
use crate::pipeline::{KantianPipeline, RiskLevel};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A contradiction between two fields in a structured document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldContradiction {
    /// JSON path of the first field (e.g., "$.risk_tolerance")
    pub field_a: String,
    /// JSON path of the second field (e.g., "$.allocation.crypto")
    pub field_b: String,
    /// Human-readable explanation of the contradiction.
    pub explanation: String,
    /// Severity of the contradiction.
    pub severity: String,
}

/// An epistemic issue found in a string field value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldEpistemicIssue {
    /// JSON path of the field.
    pub field_path: String,
    /// The string value that was analyzed.
    pub value: String,
    /// Risk level of the field content.
    pub risk_level: String,
    /// Issues detected.
    pub issues: Vec<String>,
    /// Regulated (corrected) form of the value.
    pub regulated_value: String,
}

/// Domain constraint violation found in a structured field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDomainViolation {
    pub field_path: String,
    pub value: String,
    pub constraint_id: String,
    pub message: String,
    pub regulation_reference: Option<String>,
}

/// The complete validation result for a structured decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionValidationResult {
    pub overall_risk: String,
    pub internal_contradictions: Vec<FieldContradiction>,
    pub epistemic_issues: Vec<FieldEpistemicIssue>,
    pub domain_violations: Vec<FieldDomainViolation>,
    pub auto_regulated: Vec<(String, String)>, // (field_path, new_value)
    pub summary: String,
}

/// Validates structured AI decisions against epistemic constraints.
pub struct StructuredDecisionValidator {
    pipeline: KantianPipeline,
    domain: DomainProfile,
}

impl StructuredDecisionValidator {
    pub fn new() -> Self {
        Self {
            pipeline: KantianPipeline::new(),
            domain: DomainProfile::default(),
        }
    }

    pub fn with_domain(domain_name: &str) -> Self {
        Self {
            pipeline: KantianPipeline::new(),
            domain: builtin_profile(domain_name),
        }
    }

    /// Validate a JSON string as a structured decision.
    pub fn validate_json(&self, json_str: &str) -> crate::error::Result<DecisionValidationResult> {
        let value: Value = serde_json::from_str(json_str)
            .map_err(|e| crate::error::PureReasonError::InvalidInput(e.to_string()))?;
        Ok(self.validate_value(&value, "$"))
    }

    fn validate_value(&self, value: &Value, path: &str) -> DecisionValidationResult {
        let mut contradictions = Vec::new();
        let mut epistemic_issues = Vec::new();
        let mut domain_violations = Vec::new();
        let mut auto_regulated = Vec::new();

        // Extract all string fields for epistemic analysis
        let string_fields = extract_string_fields(value, path);

        for (field_path, field_value) in &string_fields {
            if let Ok(report) = self.pipeline.process(field_value.as_str()) {
                if report.verdict.risk >= RiskLevel::Medium {
                    let issues: Vec<String> = report
                        .dialectic
                        .illusions
                        .iter()
                        .map(|illusion| format!("{:?}", illusion.kind))
                        .collect();
                    if !issues.is_empty() || report.regulated_text != *field_value {
                        epistemic_issues.push(FieldEpistemicIssue {
                            field_path: field_path.clone(),
                            value: field_value.clone(),
                            risk_level: report.verdict.risk.to_string(),
                            issues,
                            regulated_value: report.regulated_text.clone(),
                        });
                        if report.regulated_text != *field_value {
                            auto_regulated.push((field_path.clone(), report.regulated_text));
                        }
                    }
                }

                // Check domain constraints
                let dominant_cat = report.verdict.dominant_category.as_deref();
                let cat_score = report.verdict.pre_score;
                let violations = self.domain.check_all(field_value, dominant_cat, cat_score);
                for v in violations {
                    domain_violations.push(FieldDomainViolation {
                        field_path: field_path.clone(),
                        value: field_value.clone(),
                        constraint_id: v.constraint_id,
                        message: v.message,
                        regulation_reference: v.regulation_reference,
                    });
                }
            }
        }

        // Detect known structural contradictions
        contradictions.extend(detect_structural_contradictions(value, path));

        // Determine overall risk
        let overall_risk = if !contradictions.iter().any(|c| c.severity == "CRITICAL")
            && contradictions.is_empty()
            && epistemic_issues.is_empty()
            && domain_violations.is_empty()
        {
            "SAFE".to_string()
        } else if contradictions.iter().any(|c| c.severity == "CRITICAL") {
            "CRITICAL".to_string()
        } else if !contradictions.is_empty()
            || domain_violations
                .iter()
                .any(|v| v.constraint_id.contains("certain"))
        {
            "HIGH".to_string()
        } else if !epistemic_issues.is_empty() || !domain_violations.is_empty() {
            "MEDIUM".to_string()
        } else {
            "LOW".to_string()
        };

        let summary = build_summary(&contradictions, &epistemic_issues, &domain_violations);

        DecisionValidationResult {
            overall_risk,
            internal_contradictions: contradictions,
            epistemic_issues,
            domain_violations,
            auto_regulated,
            summary,
        }
    }
}

impl Default for StructuredDecisionValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract all string fields from a JSON value, with their paths.
fn extract_string_fields(value: &Value, path: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    match value {
        Value::String(s) if s.split_whitespace().count() > 3 => {
            result.push((path.to_string(), s.clone()));
        }
        Value::Object(map) => {
            for (key, val) in map {
                let child_path = format!("{}.{}", path, key);
                result.extend(extract_string_fields(val, &child_path));
            }
        }
        Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let child_path = format!("{}[{}]", path, i);
                result.extend(extract_string_fields(val, &child_path));
            }
        }
        _ => {}
    }
    result
}

/// Detect known structural contradictions in common decision schemas.
fn detect_structural_contradictions(value: &Value, _path: &str) -> Vec<FieldContradiction> {
    let mut contradictions = Vec::new();

    if let Value::Object(map) = value {
        // Medical: contraindications vs prescribed medications
        if let (Some(Value::Array(contra)), Some(Value::Array(prescribed))) =
            (map.get("contraindications"), map.get("prescribed"))
        {
            let contra_strs: Vec<String> = contra
                .iter()
                .filter_map(|v| v.as_str().map(str::to_lowercase))
                .collect();
            let prescribed_strs: Vec<String> = prescribed
                .iter()
                .filter_map(|v| v.as_str().map(str::to_lowercase))
                .collect();
            for drug in &contra_strs {
                if prescribed_strs.contains(drug) {
                    contradictions.push(FieldContradiction {
                        field_a: "$.contraindications".to_string(),
                        field_b: "$.prescribed".to_string(),
                        explanation: format!(
                            "'{}' is simultaneously contraindicated and prescribed — a CRITICAL clinical contradiction.",
                            drug
                        ),
                        severity: "CRITICAL".to_string(),
                    });
                }
            }
        }

        // Financial: conservative risk + high-volatility allocation
        if let Some(Value::String(risk_tolerance)) = map.get("risk_tolerance") {
            let tolerance_lower = risk_tolerance.to_lowercase();
            if tolerance_lower.contains("conservative") || tolerance_lower.contains("low") {
                if let Some(Value::Object(alloc_map)) =
                    map.get("recommended_allocation").or(map.get("allocation"))
                {
                    for volatile_asset in &[
                        "crypto",
                        "cryptocurrency",
                        "options",
                        "futures",
                        "leveraged",
                    ] {
                        if let Some(Value::Number(pct)) = alloc_map.get(*volatile_asset) {
                            if pct.as_f64().unwrap_or(0.0) > 0.20 {
                                contradictions.push(FieldContradiction {
                                    field_a: "$.risk_tolerance".to_string(),
                                    field_b: format!(
                                        "$.recommended_allocation.{}",
                                        volatile_asset
                                    ),
                                    explanation: format!(
                                        "Conservative risk tolerance contradicts {:.0}% allocation to high-volatility asset '{}'.",
                                        pct.as_f64().unwrap_or(0.0) * 100.0,
                                        volatile_asset
                                    ),
                                    severity: "HIGH".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Generic: conflicting boolean flags
        let yes_fields: Vec<&str> = map
            .iter()
            .filter_map(|(k, v)| {
                if v == &Value::Bool(true) {
                    Some(k.as_str())
                } else {
                    None
                }
            })
            .collect();
        let no_fields: Vec<&str> = map
            .iter()
            .filter_map(|(k, v)| {
                if v == &Value::Bool(false) {
                    Some(k.as_str())
                } else {
                    None
                }
            })
            .collect();
        for yes in &yes_fields {
            let negated = format!("no_{}", yes);
            let not_field = format!("not_{}", yes);
            if no_fields.contains(&negated.as_str()) || no_fields.contains(&not_field.as_str()) {
                contradictions.push(FieldContradiction {
                    field_a: format!("$.{}", yes),
                    field_b: format!("$.{}", negated),
                    explanation: format!(
                        "Field '{}' is true while '{}' is false — logical contradiction.",
                        yes, negated
                    ),
                    severity: "HIGH".to_string(),
                });
            }
        }

        // Recurse into nested objects
        for (key, val) in map {
            if matches!(val, Value::Object(_)) {
                contradictions.extend(detect_structural_contradictions(val, &format!("$.{}", key)));
            }
        }
    }

    contradictions
}

fn build_summary(
    contradictions: &[FieldContradiction],
    epistemic_issues: &[FieldEpistemicIssue],
    domain_violations: &[FieldDomainViolation],
) -> String {
    if contradictions.is_empty() && epistemic_issues.is_empty() && domain_violations.is_empty() {
        return "Structured decision is epistemically valid. No contradictions or violations detected.".to_string();
    }
    let mut parts = Vec::new();
    if !contradictions.is_empty() {
        parts.push(format!(
            "{} internal contradiction(s)",
            contradictions.len()
        ));
    }
    if !epistemic_issues.is_empty() {
        parts.push(format!("{} epistemic issue(s)", epistemic_issues.len()));
    }
    if !domain_violations.is_empty() {
        parts.push(format!(
            "{} domain constraint violation(s)",
            domain_violations.len()
        ));
    }
    parts.join("; ") + ". Review the structured validation report."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_json_is_safe() {
        let validator = StructuredDecisionValidator::new();
        let json = r#"{"recommendation": "Consider a balanced portfolio with bonds and equities.", "risk_level": "moderate"}"#;
        let result = validator.validate_json(json).unwrap();
        assert!(result.internal_contradictions.is_empty());
    }

    #[test]
    fn medical_contradiction_detected() {
        let validator = StructuredDecisionValidator::new();
        let json = r#"{"contraindications": ["warfarin"], "prescribed": ["aspirin", "warfarin"]}"#;
        let result = validator.validate_json(json).unwrap();
        assert!(!result.internal_contradictions.is_empty());
        assert_eq!(result.internal_contradictions[0].severity, "CRITICAL");
    }

    #[test]
    fn financial_contradiction_detected() {
        let validator = StructuredDecisionValidator::new();
        let json = r#"{"risk_tolerance": "Conservative", "recommended_allocation": {"crypto": 0.45, "bonds": 0.10}}"#;
        let result = validator.validate_json(json).unwrap();
        assert!(!result.internal_contradictions.is_empty());
    }

    #[test]
    fn invalid_json_returns_error() {
        let validator = StructuredDecisionValidator::new();
        let result = validator.validate_json("not json at all {{{");
        assert!(result.is_err());
    }
}
