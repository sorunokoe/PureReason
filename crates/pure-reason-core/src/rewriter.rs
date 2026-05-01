//! # Domain-Aware Regulative Rewriter (Solution 3, TRIZ Report V)
//!
//! Deterministic text transformation that converts constitutive (overconfident)
//! language into regulative (epistemically calibrated) language, without any LLM calls.
//!
//! ## The Kantian Basis
//!
//! Kant's Regulative/Constitutive distinction (CPR A669/B697–A702/B730):
//! - **Constitutive use:** treats an idea as denoting a real object ("The patient has cancer")
//! - **Regulative use:** uses an idea as a guiding principle ("Findings are consistent with...")
//!
//! The Regulative Rewriter applies this distinction domain-specifically,
//! converting the most common constitutive patterns per domain into their
//! structurally correct regulative counterparts.
//!
//! ## Domain Profiles
//!
//! | Domain     | Example Constitutive          | Example Regulative                        |
//! |-----------|-------------------------------|-------------------------------------------|
//! | Medical    | "has cancer"                  | "findings are consistent with malignancy" |
//! | Legal      | "is guilty"                   | "evidence suggests liability may exist"   |
//! | Financial  | "will return 12%"             | "historical patterns suggest returns of"  |
//! | Technical  | "always succeeds"             | "is designed to succeed under conditions" |
//! | General    | "must", "certainly", "always" | "may", "evidence suggests", "typically"   |
//!
//! ## No LLM Required
//!
//! All transformations are deterministic string substitutions using pre-defined
//! pattern → replacement pairs. They run in microseconds with zero token cost.

use serde::{Deserialize, Serialize};

// ─── Domain ──────────────────────────────────────────────────────────────────

/// The domain context for targeted regulative rewrites.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RewriteDomain {
    /// Medical / clinical AI output (HIPAA, FDA AI/ML guidance).
    Medical,
    /// Legal analysis and contract AI (bar standards, legal ethics).
    Legal,
    /// Financial advice and investment AI (SEC, FINRA).
    Financial,
    /// Software and technical systems (availability, security claims).
    Technical,
    /// General-purpose (catches common overconfident language in any domain).
    General,
}

impl RewriteDomain {
    /// Parse a domain string (case-insensitive) into a [`RewriteDomain`] variant.
    ///
    /// Named `parse_domain` rather than `from_str` to avoid confusion with the
    /// `std::str::FromStr` trait (which uses `Err` and is called via `str::parse()`).
    pub fn parse_domain(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "medical" | "health" | "clinical" | "hipaa" | "fda" => Self::Medical,
            "legal" | "law" | "juridical" => Self::Legal,
            "financial" | "finance" | "investment" | "sec" | "finra" => Self::Financial,
            "technical" | "tech" | "engineering" | "software" => Self::Technical,
            _ => Self::General,
        }
    }

    /// Deprecated alias for [`parse_domain`](Self::parse_domain).
    #[deprecated(
        since = "0.2.0",
        note = "use `RewriteDomain::parse_domain` — `from_str` is confusable with `std::str::FromStr::from_str`"
    )]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        Self::parse_domain(s)
    }
}

// ─── RewriteRule ─────────────────────────────────────────────────────────────

/// A single constitutive → regulative rewrite rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteRule {
    /// The pattern to find (case-insensitive substring).
    pub pattern: &'static str,
    /// The regulative replacement.
    pub replacement: &'static str,
    /// Which Kantian category is being transformed.
    pub kantian_basis: &'static str,
    /// Example of the transformation.
    pub example: &'static str,
}

// ─── DomainRewriter ──────────────────────────────────────────────────────────

/// Domain-aware regulative rewriter.
///
/// Applies the appropriate domain-specific rules first, then general rules,
/// to convert constitutive language into its regulative form.
pub struct DomainRewriter {
    domain: RewriteDomain,
}

impl DomainRewriter {
    pub fn new(domain: RewriteDomain) -> Self {
        Self { domain }
    }

    pub fn general() -> Self {
        Self {
            domain: RewriteDomain::General,
        }
    }

    /// Rewrite `text` in regulative language, returning the transformed text
    /// and a log of all substitutions made.
    pub fn rewrite(&self, text: &str) -> RewriteResult {
        let mut output = text.to_string();
        let mut applied = Vec::new();

        // Apply domain-specific rules first (higher precision)
        for rule in self.domain_rules() {
            let lower = output.to_lowercase();
            if lower.contains(rule.pattern) {
                let replaced = replace_case_insensitive(&output, rule.pattern, rule.replacement);
                if replaced != output {
                    applied.push(AppliedRule {
                        original_fragment: rule.pattern.to_string(),
                        replacement: rule.replacement.to_string(),
                        kantian_basis: rule.kantian_basis.to_string(),
                    });
                    output = replaced;
                }
            }
        }

        // Apply general rules (catch remaining overconfident language)
        for rule in GENERAL_RULES {
            let lower = output.to_lowercase();
            if lower.contains(rule.pattern) {
                let replaced = replace_case_insensitive(&output, rule.pattern, rule.replacement);
                if replaced != output {
                    applied.push(AppliedRule {
                        original_fragment: rule.pattern.to_string(),
                        replacement: rule.replacement.to_string(),
                        kantian_basis: rule.kantian_basis.to_string(),
                    });
                    output = replaced;
                }
            }
        }

        let changed = output != text;

        RewriteResult {
            original: text.to_string(),
            regulated: output,
            changed,
            rules_applied: applied,
            domain: self.domain,
        }
    }

    fn domain_rules(&self) -> &'static [RewriteRule] {
        match self.domain {
            RewriteDomain::Medical => MEDICAL_RULES,
            RewriteDomain::Legal => LEGAL_RULES,
            RewriteDomain::Financial => FINANCIAL_RULES,
            RewriteDomain::Technical => TECHNICAL_RULES,
            RewriteDomain::General => &[],
        }
    }
}

// ─── RewriteResult ───────────────────────────────────────────────────────────

/// The result of a regulative rewrite operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteResult {
    /// The original (potentially constitutive) text.
    pub original: String,
    /// The regulated text.
    pub regulated: String,
    /// Whether any changes were made.
    pub changed: bool,
    /// The list of rules that were applied.
    pub rules_applied: Vec<AppliedRule>,
    /// The domain under which the rewrite was performed.
    pub domain: RewriteDomain,
}

/// A record of a single rule application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedRule {
    pub original_fragment: String,
    pub replacement: String,
    pub kantian_basis: String,
}

// ─── Rule Tables ─────────────────────────────────────────────────────────────

/// Medical domain rules — aligned with HIPAA and FDA AI/ML guidance.
/// Necessity-modal diagnostic claims → hedged, evidence-based regulative forms.
static MEDICAL_RULES: &[RewriteRule] = &[
    RewriteRule {
        pattern: "has cancer",
        replacement: "has findings consistent with possible malignancy",
        kantian_basis:
            "Necessity → Possibility (diagnostic certainty requires histopathological confirmation)",
        example:
            "The patient has cancer → The patient has findings consistent with possible malignancy",
    },
    RewriteRule {
        pattern: "is diagnosed with",
        replacement: "presents with features consistent with",
        kantian_basis: "Existence (constitutive) → Possibility (regulative)",
        example: "is diagnosed with diabetes → presents with features consistent with diabetes",
    },
    RewriteRule {
        pattern: "will die",
        replacement: "may have a significantly reduced life expectancy",
        kantian_basis: "Necessity → Possibility (prognostic uncertainty)",
        example: "will die within 6 months → may have a significantly reduced life expectancy",
    },
    RewriteRule {
        pattern: "must take",
        replacement: "may benefit from taking",
        kantian_basis:
            "Necessity → Possibility (treatment recommendation requires physician oversight)",
        example: "must take 10mg → may benefit from taking 10mg, subject to physician review",
    },
    RewriteRule {
        pattern: "is definitely",
        replacement: "appears consistent with",
        kantian_basis: "Necessity → Possibility",
        example: "is definitely a benign lesion → appears consistent with a benign lesion",
    },
    RewriteRule {
        pattern: "requires immediate surgery",
        replacement: "may warrant urgent surgical consultation",
        kantian_basis: "Necessity → Possibility (clinical decisions require physician judgment)",
        example: "requires immediate surgery → may warrant urgent surgical consultation",
    },
];

/// Legal domain rules — aligned with bar standards and legal ethics.
static LEGAL_RULES: &[RewriteRule] = &[
    RewriteRule {
        pattern: "is guilty",
        replacement: "may bear liability based on the available evidence",
        kantian_basis: "Necessity → Possibility (guilt requires judicial determination)",
        example: "is guilty → may bear liability based on the available evidence",
    },
    RewriteRule {
        pattern: "is innocent",
        replacement: "has not been found liable on the available evidence",
        kantian_basis: "Necessity → Existence (innocence is a legal status, not a factual certainty)",
        example: "is innocent → has not been found liable on the available evidence",
    },
    RewriteRule {
        pattern: "the contract requires",
        replacement: "the contract appears to require, based on the provided clauses,",
        kantian_basis: "Necessity → Existence + attribution (legal interpretation requires authoritative reading)",
        example: "the contract requires X → the contract appears to require X, based on the provided clauses",
    },
    RewriteRule {
        pattern: "you will win",
        replacement: "available evidence suggests a favourable outcome may be possible",
        kantian_basis: "Necessity → Possibility (legal outcomes are not predictable with certainty)",
        example: "you will win this case → available evidence suggests a favourable outcome may be possible",
    },
    RewriteRule {
        pattern: "is illegal",
        replacement: "may be inconsistent with applicable law and should be reviewed by qualified counsel",
        kantian_basis: "Necessity → Possibility + defer to authority",
        example: "is illegal → may be inconsistent with applicable law",
    },
];

/// Financial domain rules — aligned with SEC Rule 10b-5 and FINRA.
static FINANCIAL_RULES: &[RewriteRule] = &[
    RewriteRule {
        pattern: "will return",
        replacement: "has historically shown returns of, though past performance does not guarantee",
        kantian_basis: "Necessity → Existence (historical) + explicit uncertainty",
        example: "will return 12% → has historically shown returns of 12%, though past performance does not guarantee",
    },
    RewriteRule {
        pattern: "is a guaranteed",
        replacement: "is presented as, though no investment is guaranteed,",
        kantian_basis: "Necessity → Possibility + disclaimer",
        example: "is a guaranteed return → is presented as a return, though no investment is guaranteed",
    },
    RewriteRule {
        pattern: "will definitely",
        replacement: "may, based on current trends,",
        kantian_basis: "Necessity → Possibility",
        example: "will definitely increase → may, based on current trends, increase",
    },
    RewriteRule {
        pattern: "the stock will",
        replacement: "based on current analysis, the stock may",
        kantian_basis: "Necessity → Possibility + evidential attribution",
        example: "the stock will rise → based on current analysis, the stock may rise",
    },
    RewriteRule {
        pattern: "is a safe investment",
        replacement: "has historically exhibited lower volatility, though all investments carry risk",
        kantian_basis: "Necessity → Existence (historical) + risk disclosure",
        example: "is a safe investment → has historically exhibited lower volatility, though all investments carry risk",
    },
];

/// Technical domain rules — aligned with software/systems reliability claims.
static TECHNICAL_RULES: &[RewriteRule] = &[
    RewriteRule {
        pattern: "always succeeds",
        replacement: "is designed to succeed under standard operating conditions",
        kantian_basis: "Totality/Necessity → Possibility (no system has absolute reliability)",
        example: "always succeeds → is designed to succeed under standard operating conditions",
    },
    RewriteRule {
        pattern: "is 100% secure",
        replacement: "implements security controls aligned with current best practices",
        kantian_basis: "Totality → Limitation (absolute security is not achievable)",
        example:
            "is 100% secure → implements security controls aligned with current best practices",
    },
    RewriteRule {
        pattern: "will never fail",
        replacement: "is engineered for high availability, with documented failure modes",
        kantian_basis: "Necessity (negation) → Possibility + transparency",
        example:
            "will never fail → is engineered for high availability, with documented failure modes",
    },
    RewriteRule {
        pattern: "is impossible to hack",
        replacement: "has not been successfully compromised in known assessments",
        kantian_basis: "Necessity → Existence (historical)",
        example:
            "is impossible to hack → has not been successfully compromised in known assessments",
    },
    RewriteRule {
        pattern: "guaranteed uptime",
        replacement: "targets uptime of",
        kantian_basis: "Necessity → Possibility (SLAs are targets, not guarantees)",
        example: "guaranteed uptime of 99.9% → targets uptime of 99.9%",
    },
];

/// General rules — applied in every domain as a final catch-all.
/// Targets the most common certainty overreach markers in natural language.
static GENERAL_RULES: &[RewriteRule] = &[
    RewriteRule {
        pattern: "it is certain that",
        replacement: "evidence suggests that",
        kantian_basis: "Necessity → Existence",
        example: "it is certain that X → evidence suggests that X",
    },
    RewriteRule {
        pattern: "is certainly",
        replacement: "appears to be",
        kantian_basis: "Necessity → Existence",
        example: "is certainly true → appears to be true",
    },
    RewriteRule {
        pattern: "without a doubt",
        replacement: "with reasonable confidence",
        kantian_basis: "Necessity → Possibility",
        example: "true without a doubt → true with reasonable confidence",
    },
    RewriteRule {
        pattern: "i am absolutely sure",
        replacement: "based on available information, it appears",
        kantian_basis: "Necessity → Existence + epistemic attribution",
        example: "I am absolutely sure → Based on available information, it appears",
    },
    RewriteRule {
        pattern: "this is definitely",
        replacement: "this appears to be",
        kantian_basis: "Necessity → Existence",
        example: "this is definitely the cause → this appears to be the cause",
    },
    RewriteRule {
        pattern: "there is no doubt",
        replacement: "evidence strongly suggests",
        kantian_basis: "Necessity → Existence",
        example: "there is no doubt that → evidence strongly suggests that",
    },
    RewriteRule {
        pattern: "is the only explanation",
        replacement: "is a leading explanation among those currently considered",
        kantian_basis: "Unity (exclusive) → Plurality (open set of explanations)",
        example:
            "is the only explanation → is a leading explanation among those currently considered",
    },
    RewriteRule {
        pattern: "will always",
        replacement: "typically",
        kantian_basis: "Totality → Plurality",
        example: "will always work → typically works",
    },
    RewriteRule {
        pattern: "will never",
        replacement: "rarely",
        kantian_basis: "Totality (negation) → Limitation",
        example: "will never fail → rarely fails",
    },
];

// ─── String manipulation helper ───────────────────────────────────────────────

/// Case-insensitive substring replacement (preserves surrounding case of non-matched text).
fn replace_case_insensitive(text: &str, pattern: &str, replacement: &str) -> String {
    let lower = text.to_lowercase();
    let lower_pattern = pattern.to_lowercase();

    if let Some(pos) = lower.find(&lower_pattern) {
        let before = &text[..pos];
        let after = &text[pos + pattern.len()..];
        format!("{}{}{}", before, replacement, after)
    } else {
        text.to_string()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn medical_rewriter_transforms_diagnosis() {
        let rewriter = DomainRewriter::new(RewriteDomain::Medical);
        let result = rewriter.rewrite("The patient has cancer and must take 10mg daily.");
        assert!(
            result.changed,
            "Medical constitutive claim should be rewritten"
        );
        assert!(
            result.regulated.contains("consistent with"),
            "Should use regulative language"
        );
        assert!(
            !result.regulated.to_lowercase().contains("has cancer"),
            "Constitutive diagnosis should be replaced"
        );
    }

    #[test]
    fn financial_rewriter_transforms_guarantee() {
        let rewriter = DomainRewriter::new(RewriteDomain::Financial);
        let result = rewriter.rewrite("This investment will return 15% annually.");
        assert!(result.changed);
        assert!(
            !result.regulated.to_lowercase().contains("will return"),
            "Future certainty should be rewritten"
        );
    }

    #[test]
    fn legal_rewriter_transforms_guilt() {
        let rewriter = DomainRewriter::new(RewriteDomain::Legal);
        let result = rewriter.rewrite("The defendant is guilty of fraud.");
        assert!(result.changed);
        assert!(
            !result.regulated.to_lowercase().contains("is guilty"),
            "Guilt determination should be rewritten"
        );
    }

    #[test]
    fn technical_rewriter_transforms_certainty() {
        let rewriter = DomainRewriter::new(RewriteDomain::Technical);
        let result = rewriter.rewrite("Our system will never fail and is 100% secure.");
        assert!(result.changed);
        assert!(
            !result.regulated.to_lowercase().contains("will never fail"),
            "Absolute reliability claim should be rewritten"
        );
    }

    #[test]
    fn general_rules_catch_certainty_language() {
        let rewriter = DomainRewriter::general();
        let result = rewriter.rewrite("It is certain that the economy will grow.");
        assert!(result.changed);
        assert!(
            result.regulated.contains("suggests"),
            "Should use evidential language"
        );
    }

    #[test]
    fn safe_text_unchanged() {
        let rewriter = DomainRewriter::general();
        let text = "Evidence suggests the economy may recover in the coming quarters.";
        let result = rewriter.rewrite(text);
        assert!(
            !result.changed,
            "Already-regulative text should not be changed"
        );
        assert_eq!(result.regulated, text);
    }

    #[test]
    fn domain_from_str() {
        assert_eq!(
            RewriteDomain::parse_domain("medical"),
            RewriteDomain::Medical
        );
        assert_eq!(
            RewriteDomain::parse_domain("financial"),
            RewriteDomain::Financial
        );
        assert_eq!(RewriteDomain::parse_domain("legal"), RewriteDomain::Legal);
        assert_eq!(
            RewriteDomain::parse_domain("technical"),
            RewriteDomain::Technical
        );
        assert_eq!(RewriteDomain::parse_domain("other"), RewriteDomain::General);
    }

    #[test]
    fn rules_applied_logged() {
        let rewriter = DomainRewriter::new(RewriteDomain::Medical);
        let result = rewriter.rewrite("The patient has cancer.");
        assert!(
            !result.rules_applied.is_empty(),
            "Applied rules should be logged"
        );
        assert!(result.rules_applied[0].kantian_basis.contains("Necessity"));
    }
}
