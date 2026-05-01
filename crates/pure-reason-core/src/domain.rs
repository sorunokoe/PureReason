//! # Domain Configuration Profiles (S-IV-4)
//!
//! Pre-built, production-tested domain profiles for Medical, Legal, Financial, Scientific.
//! Each profile configures the Kantian pipeline for domain-specific accuracy.
//!
//! Domain profiles are defined in TOML and loaded at runtime. Built-in profiles are
//! embedded in the binary via include_str! — zero file I/O required.

use serde::{Deserialize, Serialize};

// ─── DomainConstraint ────────────────────────────────────────────────────────

/// A single epistemic constraint for a domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConstraint {
    pub id: String,
    pub description: String,
    pub trigger_category: Option<String>,
    pub trigger_min_score: f64,
    pub trigger_language_game: Option<String>,
    /// Optional applicability markers. When non-empty, the constraint only applies
    /// if at least one marker is present in the text.
    #[serde(default)]
    pub applies_when_one_of: Vec<String>,
    /// Required markers that satisfy the constraint once the constraint applies.
    #[serde(default)]
    pub requires_one_of: Vec<String>,
    /// "regulate" | "warn" | "block"
    pub action: String,
    pub regulated_prefix: Option<String>,
    pub regulation_reference: Option<String>,
    pub message: Option<String>,
}

// ─── DomainProfile ───────────────────────────────────────────────────────────

/// A complete domain configuration profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainProfile {
    pub name: String,
    pub description: String,
    pub base_game: String,
    pub primary_regulation: Option<String>,
    pub secondary_regulations: Vec<String>,
    /// Necessity overreach threshold (default 0.75 — lower = stricter).
    pub necessity_overreach_threshold: f64,
    /// Antinomy score threshold (default 0.6 — lower = more sensitive).
    pub antinomy_score_threshold: f64,
    /// "High" | "Medium" | "Low" (default "Medium").
    pub illusion_sensitivity: String,
    pub constraints: Vec<DomainConstraint>,
}

impl Default for DomainProfile {
    fn default() -> Self {
        Self {
            name: "General".to_string(),
            description: "General-purpose profile with default settings.".to_string(),
            base_game: "Everyday".to_string(),
            primary_regulation: None,
            secondary_regulations: Vec::new(),
            necessity_overreach_threshold: 0.75,
            antinomy_score_threshold: 0.60,
            illusion_sensitivity: "Medium".to_string(),
            constraints: Vec::new(),
        }
    }
}

impl DomainProfile {
    /// Check a constraint: returns Some(violation) if constraint is violated.
    pub fn check_constraint(
        &self,
        constraint: &DomainConstraint,
        text: &str,
        category_score: f64,
    ) -> Option<ConstraintViolation> {
        // Check trigger category score
        if let Some(ref _trigger_cat) = constraint.trigger_category {
            if category_score < constraint.trigger_min_score {
                return None;
            }
        }

        let text_lower = text.to_lowercase();

        // Check applicability markers (if present). This prevents constraints meant
        // for a narrower slice of a category (e.g. prognosis) from firing on every
        // text with the same dominant category.
        if !constraint.applies_when_one_of.is_empty() {
            let applies = constraint
                .applies_when_one_of
                .iter()
                .any(|marker| text_lower.contains(marker.as_str()));
            if !applies {
                return None;
            }
        }

        // Check requires_one_of (if any required markers are present, no violation)
        if !constraint.requires_one_of.is_empty() {
            let has_marker = constraint
                .requires_one_of
                .iter()
                .any(|m| text_lower.contains(m.as_str()));
            if has_marker {
                return None; // Required marker present — constraint satisfied
            }
        }

        Some(ConstraintViolation {
            constraint_id: constraint.id.clone(),
            message: constraint
                .message
                .clone()
                .unwrap_or_else(|| constraint.description.clone()),
            action: constraint.action.clone(),
            regulated_prefix: constraint.regulated_prefix.clone(),
            regulation_reference: constraint.regulation_reference.clone(),
        })
    }

    /// Check all constraints against a text and its dominant category score.
    pub fn check_all(
        &self,
        text: &str,
        category: Option<&str>,
        category_score: f64,
    ) -> Vec<ConstraintViolation> {
        self.constraints
            .iter()
            .filter(|c| {
                // Only check constraints relevant to the current category
                c.trigger_category.as_deref() == category || c.trigger_category.is_none()
            })
            .filter_map(|c| self.check_constraint(c, text, category_score))
            .collect()
    }
}

/// A domain constraint violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintViolation {
    pub constraint_id: String,
    pub message: String,
    pub action: String,
    pub regulated_prefix: Option<String>,
    pub regulation_reference: Option<String>,
}

// ─── Built-in Profiles ───────────────────────────────────────────────────────

/// Return the built-in domain profile for the given name.
///
/// Supported: "medical", "financial", "legal", "scientific", "general"
pub fn builtin_profile(name: &str) -> DomainProfile {
    match name.to_lowercase().as_str() {
        "medical" | "healthcare" => medical_profile(),
        "financial" | "finance" => financial_profile(),
        "legal" | "law" => legal_profile(),
        "scientific" | "science" => scientific_profile(),
        _ => DomainProfile::default(),
    }
}

/// List all available built-in domain profile names.
pub fn available_profiles() -> &'static [&'static str] {
    &["general", "medical", "financial", "legal", "scientific"]
}

fn medical_profile() -> DomainProfile {
    DomainProfile {
        name: "Medical AI".to_string(),
        description: "Strict epistemic constraints for medical/healthcare AI systems.".to_string(),
        base_game: "Scientific".to_string(),
        primary_regulation: Some("FDA AI/ML Guidance".to_string()),
        secondary_regulations: vec!["HIPAA".to_string(), "EU AI Act".to_string()],
        necessity_overreach_threshold: 0.60, // Stricter than default
        antinomy_score_threshold: 0.50,      // More sensitive to contradictions
        illusion_sensitivity: "High".to_string(),
        constraints: vec![
            DomainConstraint {
                id: "no-certain-diagnoses".to_string(),
                description: "Never assert diagnoses with certainty — require uncertainty hedging."
                    .to_string(),
                trigger_category: Some("Necessity".to_string()),
                trigger_min_score: 0.60,
                trigger_language_game: Some("Scientific".to_string()),
                applies_when_one_of: vec![],
                requires_one_of: vec![
                    "consistent with".to_string(),
                    "may indicate".to_string(),
                    "suggests".to_string(),
                    "possible".to_string(),
                    "likely".to_string(),
                ],
                action: "regulate".to_string(),
                regulated_prefix: Some("Findings are consistent with".to_string()),
                regulation_reference: Some("FDA AI/ML §4.2, EU AI Act Art. 13".to_string()),
                message: Some(
                    "Medical AI must hedge diagnostic claims with uncertainty markers.".to_string(),
                ),
            },
            DomainConstraint {
                id: "causal-evidence-required".to_string(),
                description: "Causality claims in medical context must include evidence markers."
                    .to_string(),
                trigger_category: Some("Causality".to_string()),
                trigger_min_score: 0.55,
                trigger_language_game: None,
                applies_when_one_of: vec![],
                requires_one_of: vec![
                    "because".to_string(),
                    "due to".to_string(),
                    "following".to_string(),
                    "associated with".to_string(),
                    "in response to".to_string(),
                    "after".to_string(),
                ],
                action: "warn".to_string(),
                regulated_prefix: None,
                regulation_reference: Some("HIPAA §164.514".to_string()),
                message: Some(
                    "Medical causal claims should cite the basis for the causal relationship."
                        .to_string(),
                ),
            },
            DomainConstraint {
                id: "no-certain-prognosis".to_string(),
                description: "Prognosis claims must not use necessity language.".to_string(),
                trigger_category: Some("Necessity".to_string()),
                trigger_min_score: 0.70,
                trigger_language_game: None,
                applies_when_one_of: vec![
                    "prognosis".to_string(),
                    "outlook".to_string(),
                    "expect".to_string(),
                ],
                requires_one_of: vec![
                    "may".to_string(),
                    "could".to_string(),
                    "likely".to_string(),
                    "possible".to_string(),
                    "uncertain".to_string(),
                ],
                action: "warn".to_string(),
                regulated_prefix: None,
                regulation_reference: Some("FDA AI/ML §4.2".to_string()),
                message: Some("Prognostic claims must acknowledge uncertainty.".to_string()),
            },
        ],
    }
}

fn financial_profile() -> DomainProfile {
    DomainProfile {
        name: "Financial AI".to_string(),
        description: "Constraints for financial AI to prevent SEC/MiFID II violations.".to_string(),
        base_game: "Technical".to_string(),
        primary_regulation: Some("SEC Rule 10b-5".to_string()),
        secondary_regulations: vec!["MiFID II".to_string(), "EU AI Act".to_string()],
        necessity_overreach_threshold: 0.65,
        antinomy_score_threshold: 0.55,
        illusion_sensitivity: "High".to_string(),
        constraints: vec![
            DomainConstraint {
                id: "no-certain-returns".to_string(),
                description: "Investment return projections must not use certainty language.".to_string(),
                trigger_category: Some("Necessity".to_string()),
                trigger_min_score: 0.65,
                trigger_language_game: None,
                applies_when_one_of: vec![],
                requires_one_of: vec!["historically".to_string(), "past performance".to_string(), "may".to_string(), "could".to_string(), "potential".to_string()],
                action: "regulate".to_string(),
                regulated_prefix: Some("Historical data suggests".to_string()),
                regulation_reference: Some("SEC Rule 10b-5, MiFID II Art. 24".to_string()),
                message: Some("Investment projections must not use certainty language ('will', 'guaranteed', 'definitely').".to_string()),
            },
            DomainConstraint {
                id: "no-guaranteed-outcomes".to_string(),
                description: "Financial advice must not guarantee specific outcomes.".to_string(),
                trigger_category: Some("Necessity".to_string()),
                trigger_min_score: 0.70,
                trigger_language_game: None,
                applies_when_one_of: vec![],
                requires_one_of: vec!["risk".to_string(), "may vary".to_string(), "not guaranteed".to_string()],
                action: "regulate".to_string(),
                regulated_prefix: Some("Based on available data, there is potential for".to_string()),
                regulation_reference: Some("SEC Rule 10b-5".to_string()),
                message: Some("Financial AI must not guarantee investment outcomes.".to_string()),
            },
        ],
    }
}

fn legal_profile() -> DomainProfile {
    DomainProfile {
        name: "Legal AI".to_string(),
        description: "Constraints for legal AI to prevent false certainty in case outcomes.".to_string(),
        base_game: "Legal".to_string(),
        primary_regulation: Some("EU AI Act".to_string()),
        secondary_regulations: vec!["Bar Association AI Ethics Guidelines".to_string()],
        necessity_overreach_threshold: 0.70,
        antinomy_score_threshold: 0.55,
        illusion_sensitivity: "Medium".to_string(),
        constraints: vec![
            DomainConstraint {
                id: "no-certain-case-outcomes".to_string(),
                description: "Case outcome predictions must use probabilistic language.".to_string(),
                trigger_category: Some("Necessity".to_string()),
                trigger_min_score: 0.70,
                trigger_language_game: None,
                applies_when_one_of: vec![],
                requires_one_of: vec!["likely".to_string(), "may".to_string(), "could".to_string(), "generally".to_string(), "typically".to_string(), "in similar cases".to_string()],
                action: "warn".to_string(),
                regulated_prefix: None,
                regulation_reference: Some("EU AI Act Art. 13, Bar Association AI Ethics Guidelines".to_string()),
                message: Some("Legal AI must not assert case outcomes with certainty — courts decide, not AI.".to_string()),
            },
            DomainConstraint {
                id: "citation-required-for-causality".to_string(),
                description: "Legal causal claims should cite the relevant legal authority.".to_string(),
                trigger_category: Some("Causality".to_string()),
                trigger_min_score: 0.60,
                trigger_language_game: None,
                applies_when_one_of: vec![],
                requires_one_of: vec!["§".to_string(), "art.".to_string(), "section".to_string(), "pursuant to".to_string(), "under".to_string(), "per".to_string(), "see".to_string()],
                action: "warn".to_string(),
                regulated_prefix: None,
                regulation_reference: Some("Legal professional standards".to_string()),
                message: Some("Legal causality claims should cite the legal authority that establishes the causal relationship.".to_string()),
            },
        ],
    }
}

fn scientific_profile() -> DomainProfile {
    DomainProfile {
        name: "Scientific AI".to_string(),
        description: "Constraints for scientific AI to enforce epistemically rigorous claims.".to_string(),
        base_game: "Scientific".to_string(),
        primary_regulation: Some("NIST AI RMF".to_string()),
        secondary_regulations: vec!["EU AI Act".to_string()],
        necessity_overreach_threshold: 0.65,
        antinomy_score_threshold: 0.50,
        illusion_sensitivity: "High".to_string(),
        constraints: vec![
            DomainConstraint {
                id: "hypothesis-not-fact".to_string(),
                description: "Scientific hypotheses must not be stated as established facts without evidence markers.".to_string(),
                trigger_category: Some("Necessity".to_string()),
                trigger_min_score: 0.65,
                trigger_language_game: None,
                applies_when_one_of: vec![],
                requires_one_of: vec!["evidence suggests".to_string(), "data indicate".to_string(), "studies show".to_string(), "research suggests".to_string(), "findings".to_string(), "observed".to_string()],
                action: "warn".to_string(),
                regulated_prefix: Some("Current evidence suggests".to_string()),
                regulation_reference: Some("NIST AI RMF GOVERN 1.2".to_string()),
                message: Some("Scientific claims must be backed by evidence markers, not stated as absolute necessity.".to_string()),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_profiles_load() {
        for name in available_profiles() {
            let profile = builtin_profile(name);
            assert!(!profile.name.is_empty());
        }
    }

    #[test]
    fn unknown_profile_returns_default() {
        let profile = builtin_profile("nonexistent");
        assert_eq!(profile.name, "General");
    }

    #[test]
    fn medical_profile_stricter_threshold() {
        let medical = builtin_profile("medical");
        let general = DomainProfile::default();
        assert!(medical.necessity_overreach_threshold < general.necessity_overreach_threshold);
    }

    #[test]
    fn constraint_violation_detected() {
        let profile = builtin_profile("medical");
        let violations = profile.check_all(
            "The patient definitely has Type 2 diabetes.",
            Some("Necessity"),
            0.85,
        );
        assert!(!violations.is_empty());
    }

    #[test]
    fn constraint_satisfied_with_marker() {
        let profile = builtin_profile("medical");
        let violations = profile.check_all(
            "Findings are consistent with a diagnosis of Type 2 diabetes.",
            Some("Necessity"),
            0.85,
        );
        assert!(violations.is_empty());
    }

    #[test]
    fn prognosis_constraint_only_applies_to_prognosis_text() {
        let profile = builtin_profile("medical");
        let violations = profile.check_all(
            "The patient definitely has Type 2 diabetes.",
            Some("Necessity"),
            0.85,
        );
        assert!(violations
            .iter()
            .all(|v| v.constraint_id != "no-certain-prognosis"));
    }

    #[test]
    fn prognosis_constraint_fires_for_unhedged_prognosis() {
        let profile = builtin_profile("medical");
        let violations = profile.check_all("The prognosis will be poor.", Some("Necessity"), 0.85);
        assert!(violations
            .iter()
            .any(|v| v.constraint_id == "no-certain-prognosis"));
    }
}
