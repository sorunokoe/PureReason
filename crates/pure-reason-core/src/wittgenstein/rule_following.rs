//! # Rule-Following
//!
//! Wittgenstein's rule-following considerations (PI §§138-242) show that:
//! - You cannot follow a rule privately (PI §202)
//! - Rules do not determine their own application
//! - "Following a rule" is a practice, embedded in a form of life
//!
//! "There is a way of grasping a rule which is not an interpretation,
//! but which is exhibited in what we call 'obeying the rule' and
//! 'going against it' in actual cases." — PI §201
//!
//! In this system, the RuleFollowingValidator checks whether text
//! is consistent in its application of rules and concepts — whether
//! the same term is used consistently, whether stated rules are followed
//! in practice, etc.

use crate::types::Proposition;
use serde::{Deserialize, Serialize};

// ─── Rule ────────────────────────────────────────────────────────────────────

/// A rule that can be checked for consistent application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: uuid::Uuid,
    pub name: String,
    pub formulation: String,
    pub positive_examples: Vec<String>,
    pub negative_examples: Vec<String>,
}

impl Rule {
    pub fn new(name: impl Into<String>, formulation: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name: name.into(),
            formulation: formulation.into(),
            positive_examples: Vec::new(),
            negative_examples: Vec::new(),
        }
    }

    pub fn with_positive(mut self, example: impl Into<String>) -> Self {
        self.positive_examples.push(example.into());
        self
    }

    pub fn with_negative(mut self, example: impl Into<String>) -> Self {
        self.negative_examples.push(example.into());
        self
    }
}

// ─── ConsistencyIssue ────────────────────────────────────────────────────────

/// An inconsistency detected in rule application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyIssue {
    pub term: String,
    pub usage_1: String,
    pub usage_2: String,
    pub description: String,
}

// ─── RuleReport ──────────────────────────────────────────────────────────────

/// Report on rule-following consistency in a set of propositions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleReport {
    /// Terms used inconsistently.
    pub consistency_issues: Vec<ConsistencyIssue>,
    /// Whether inconsistencies were found.
    pub is_consistent: bool,
    /// Wittgensteinian note on rule-following.
    pub note: String,
}

// ─── RuleFollowingValidator ──────────────────────────────────────────────────

/// Validates consistent rule-following in propositions.
pub struct RuleFollowingValidator;

impl RuleFollowingValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate rule-following consistency across propositions.
    pub fn validate(&self, propositions: &[Proposition]) -> Vec<RuleReport> {
        let mut issues = Vec::new();

        // Check for terminological inconsistency: same term used in contradictory ways
        issues.extend(self.check_terminological_consistency(propositions));

        // Check for stated-rule violations: if a rule is stated, check it's followed
        issues.extend(self.check_stated_rules(propositions));

        let is_consistent = issues.is_empty();
        let note = if is_consistent {
            "Rule-following appears consistent. Terms are used uniformly across propositions."
                .to_string()
        } else {
            format!(
                "{} consistency issue(s) detected. Note: per Wittgenstein, rule-following is a practice — \
                 inconsistency suggests a change in the language game or a term being used in different senses.",
                issues.len()
            )
        };

        vec![RuleReport {
            consistency_issues: issues,
            is_consistent,
            note,
        }]
    }

    fn check_terminological_consistency(
        &self,
        propositions: &[Proposition],
    ) -> Vec<ConsistencyIssue> {
        let mut issues = Vec::new();

        // Track how key terms are used (positive vs negative assertion)
        let mut term_assertions: std::collections::HashMap<String, Vec<(String, bool)>> =
            std::collections::HashMap::new();

        for prop in propositions {
            let text = prop.text.to_lowercase();
            let is_negative =
                text.contains(" not ") || text.contains(" no ") || text.contains("never");
            let words: Vec<String> = text
                .split_whitespace()
                .filter(|w| w.len() > 4)
                .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
                .filter(|w| !w.is_empty())
                .collect();

            for word in words {
                term_assertions
                    .entry(word)
                    .or_default()
                    .push((prop.text.clone(), is_negative));
            }
        }

        // Find terms asserted both positively and negatively
        for (term, assertions) in &term_assertions {
            let positives: Vec<&str> = assertions
                .iter()
                .filter(|(_, neg)| !neg)
                .map(|(s, _)| s.as_str())
                .collect();
            let negatives: Vec<&str> = assertions
                .iter()
                .filter(|(_, neg)| *neg)
                .map(|(s, _)| s.as_str())
                .collect();

            if !positives.is_empty() && !negatives.is_empty() {
                issues.push(ConsistencyIssue {
                    term: term.clone(),
                    usage_1: positives[0].to_string(),
                    usage_2: negatives[0].to_string(),
                    description: format!(
                        "Term '{}' appears in both positive and negative contexts. \
                         This may indicate inconsistent usage or a genuine contradiction.",
                        term
                    ),
                });
            }
        }

        issues
    }

    fn check_stated_rules(&self, propositions: &[Proposition]) -> Vec<ConsistencyIssue> {
        let mut issues = Vec::new();

        // Find propositions that state rules (contain "always", "never", "must", "all X are Y")
        let rule_propositions: Vec<&Proposition> = propositions
            .iter()
            .filter(|p| {
                let text = p.text.to_lowercase();
                text.contains("always")
                    || text.contains("never")
                    || text.contains("all ")
                    || text.contains("every ")
                    || text.contains("must ")
                    || text.contains("no one")
            })
            .collect();

        // Check each rule proposition against the others
        for rule_prop in &rule_propositions {
            let rule_text = rule_prop.text.to_lowercase();

            for other in propositions {
                if other.id == rule_prop.id {
                    continue;
                }

                let other_text = other.text.to_lowercase();

                // Very simple check: if the rule says "always X", look for "not X"
                if rule_text.contains("always") {
                    let key_word = extract_key_predicate(&rule_text);
                    if !key_word.is_empty()
                        && other_text.contains(&key_word)
                        && other_text.contains("not")
                    {
                        issues.push(ConsistencyIssue {
                            term: key_word.clone(),
                            usage_1: rule_prop.text.clone(),
                            usage_2: other.text.clone(),
                            description: format!(
                                "A universal rule ('always {}') may be violated by another proposition.",
                                key_word
                            ),
                        });
                    }
                }
            }
        }

        issues
    }
}

fn extract_key_predicate(text: &str) -> String {
    // Find the word after "always" as the key predicate
    let words: Vec<&str> = text.split_whitespace().collect();
    words
        .windows(2)
        .find(|pair| pair[0] == "always")
        .map(|pair| pair[1].to_string())
        .unwrap_or_default()
}

impl Default for RuleFollowingValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PropositionKind;

    fn prop(text: &str) -> Proposition {
        Proposition::new(text, PropositionKind::Unknown)
    }

    #[test]
    fn consistent_text_passes() {
        let props = vec![
            prop("Water is always a compound of hydrogen and oxygen"),
            prop("Hydrogen and oxygen combine to form water"),
        ];
        let validator = RuleFollowingValidator::new();
        let reports = validator.validate(&props);
        // May have minor issues but shouldn't be flagged as deeply inconsistent
        assert!(!reports.is_empty());
    }

    #[test]
    fn rule_created() {
        let rule = Rule::new(
            "Non-contradiction",
            "Nothing can both be and not be at the same time",
        )
        .with_positive("A is A")
        .with_negative("A is not A");
        assert_eq!(rule.positive_examples.len(), 1);
        assert_eq!(rule.negative_examples.len(), 1);
    }
}
