//! # Validate-Decision Command (S-IV-9)
//!
//! Validate a JSON decision against Kantian epistemic and domain constraints.
//!
//! ```
//! pure-reason validate-decision '{"risk_tolerance":"Conservative","recommended_allocation":{"crypto":0.45}}'
//! pure-reason validate-decision '{"contraindications":["warfarin"],"prescribed":["warfarin"]}' --domain medical
//! ```

use anyhow::Result;
use clap::Args;
use colored::Colorize;
use pure_reason_core::structured_validator::StructuredDecisionValidator;

#[derive(Args)]
pub struct ValidateDecisionCmd {
    /// JSON string to validate
    pub json: String,

    /// Domain profile to apply (e.g. medical, financial)
    #[arg(long, default_value = "")]
    pub domain: String,
}

impl ValidateDecisionCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        let validator = if self.domain.is_empty() {
            StructuredDecisionValidator::new()
        } else {
            StructuredDecisionValidator::with_domain(&self.domain)
        };

        let result = validator
            .validate_json(&self.json)
            .map_err(|e| anyhow::anyhow!(e))?;

        match format {
            "json" => println!("{}", serde_json::to_string_pretty(&result)?),
            "markdown" => {
                println!("# Structured Decision Validation\n");
                println!("**Overall Risk:** {}", result.overall_risk);
                println!("**Summary:** {}\n", result.summary);

                if !result.internal_contradictions.is_empty() {
                    println!("## Internal Contradictions\n");
                    for c in &result.internal_contradictions {
                        println!(
                            "- **[{}]** `{}` ↔ `{}`: {}",
                            c.severity, c.field_a, c.field_b, c.explanation
                        );
                    }
                    println!();
                }
                if !result.epistemic_issues.is_empty() {
                    println!("## Epistemic Issues\n");
                    for e in &result.epistemic_issues {
                        println!(
                            "- `{}` [{}]: {}",
                            e.field_path,
                            e.risk_level,
                            e.issues.join(", ")
                        );
                    }
                    println!();
                }
                if !result.domain_violations.is_empty() {
                    println!("## Domain Violations\n");
                    for v in &result.domain_violations {
                        println!("- `{}`: {} ({})", v.field_path, v.message, v.constraint_id);
                    }
                    println!();
                }
                if !result.auto_regulated.is_empty() {
                    println!("## Auto-Regulated Fields\n");
                    for (path, new_val) in &result.auto_regulated {
                        println!("- `{}` → `{}`", path, new_val);
                    }
                }
            }
            _ => print_plain(&result),
        }

        Ok(())
    }
}

fn print_plain(result: &pure_reason_core::structured_validator::DecisionValidationResult) {
    let divider = "─".repeat(60);
    println!("{}", divider.dimmed());
    println!("{}", "  STRUCTURED DECISION VALIDATION".bold().cyan());
    println!("{}", divider.dimmed());

    let risk_colored = match result.overall_risk.as_str() {
        "SAFE" => result.overall_risk.green().bold().to_string(),
        "LOW" => result.overall_risk.yellow().to_string(),
        "MEDIUM" | "HIGH" => result.overall_risk.red().bold().to_string(),
        "CRITICAL" => result.overall_risk.red().bold().to_string(),
        _ => result.overall_risk.clone(),
    };
    println!("{} {}", "Overall Risk:".bold(), risk_colored);
    println!("{} {}", "Summary:".bold(), result.summary);
    println!();

    if result.internal_contradictions.is_empty()
        && result.epistemic_issues.is_empty()
        && result.domain_violations.is_empty()
    {
        println!(
            "  {} No contradictions or violations detected.",
            "✓".green().bold()
        );
    } else {
        if !result.internal_contradictions.is_empty() {
            println!("{}", "  Internal Contradictions:".bold().red());
            for c in &result.internal_contradictions {
                println!(
                    "    {} [{}] {} ↔ {}",
                    "✗".red(),
                    c.severity.red(),
                    c.field_a.italic(),
                    c.field_b.italic()
                );
                println!("      {}", c.explanation.dimmed());
            }
            println!();
        }
        if !result.epistemic_issues.is_empty() {
            println!("{}", "  Epistemic Issues:".bold().yellow());
            for e in &result.epistemic_issues {
                println!(
                    "    {} {} [{}]",
                    "⚠".yellow(),
                    e.field_path.italic(),
                    e.risk_level
                );
                println!("      {}", e.issues.join(", ").dimmed());
            }
            println!();
        }
        if !result.domain_violations.is_empty() {
            println!("{}", "  Domain Violations:".bold().yellow());
            for v in &result.domain_violations {
                println!(
                    "    {} {}: {}",
                    "⚠".yellow(),
                    v.field_path.italic(),
                    v.message.dimmed()
                );
            }
            println!();
        }
        if !result.auto_regulated.is_empty() {
            println!("{}", "  Auto-Regulated:".bold().green());
            for (path, new_val) in &result.auto_regulated {
                println!(
                    "    {} {} → {}",
                    "→".green(),
                    path.italic(),
                    new_val.dimmed()
                );
            }
            println!();
        }
    }

    println!("{}", divider.dimmed());
}
