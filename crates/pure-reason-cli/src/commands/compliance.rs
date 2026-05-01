//! # Compliance Command (S-IV-1)
//!
//! Run a regulatory compliance check on text against a chosen framework.
//!
//! ```
//! pure-reason compliance "God definitely exists." --framework eu-ai-act
//! pure-reason compliance "The drug will cure you." --framework hipaa
//! ```

use super::get_text;
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use pure_reason_core::{
    compliance::{ComplianceFramework, ComplianceReport},
    pipeline::KantianPipeline,
};

#[derive(Args)]
pub struct ComplianceCmd {
    /// Text to check (reads from stdin if omitted)
    pub text: Option<String>,

    /// Regulatory framework to evaluate against
    #[arg(long, default_value = "eu-ai-act",
          value_parser = ["eu-ai-act", "hipaa", "sec-rule-10b5", "fda-ai-ml", "nist-ai-rmf", "gdpr"])]
    pub framework: String,
}

impl ComplianceCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        let text = get_text(&self.text)?;
        let framework = parse_framework(&self.framework);

        let pipeline = KantianPipeline::new();
        let report = pipeline.process(&text).map_err(|e| anyhow::anyhow!(e))?;
        let compliance = ComplianceReport::generate(&report, framework);

        match format {
            "json" => println!("{}", serde_json::to_string_pretty(&compliance)?),
            "markdown" => {
                println!("# Compliance Report — {}\n", compliance.framework);
                println!("**Status:** {}", compliance.overall_status);
                println!("**Findings:** {}", compliance.findings.len());
                println!("**Auto-remediated:** {}", compliance.auto_remediated);
                println!("**Audit hash:** `{}`", compliance.audit_hash);
                println!("**Issued:** {}\n", compliance.issued_at);
                if !compliance.findings.is_empty() {
                    println!("## Findings\n");
                    for (i, f) in compliance.findings.iter().enumerate() {
                        println!("### Finding {} — {} [{}]", i + 1, f.article, f.severity);
                        println!("- **Violation:** {}", f.violation_type);
                        println!("- **Evidence:** {}", f.evidence);
                        println!("- **Remedy:** {}\n", f.remediation_hint);
                    }
                }
            }
            _ => {
                let display = compliance.display();
                // Colorise the status line
                let colored = display
                    .replace("NON-COMPLIANT", &"NON-COMPLIANT".red().bold().to_string())
                    .replace("COMPLIANT", &"COMPLIANT".green().bold().to_string())
                    .replace(
                        "REQUIRES REVIEW",
                        &"REQUIRES REVIEW".yellow().bold().to_string(),
                    );
                print!("{}", colored);
            }
        }

        Ok(())
    }
}

fn parse_framework(s: &str) -> ComplianceFramework {
    match s {
        "hipaa" => ComplianceFramework::Hipaa,
        "sec-rule-10b5" => ComplianceFramework::SecRule10b5,
        "fda-ai-ml" => ComplianceFramework::FdaAiMlGuidance,
        "nist-ai-rmf" => ComplianceFramework::NistAiRmf,
        "gdpr" => ComplianceFramework::Gdpr,
        _ => ComplianceFramework::EuAiAct,
    }
}
