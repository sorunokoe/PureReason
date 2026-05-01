use super::get_text;
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Table};
use pure_reason_core::claims::{annotate_claims, annotation_to_triple, ClaimAnnotatedReport};
use pure_reason_core::pipeline::RiskLevel;

/// Annotate each sentence with its own epistemic risk level (TRIZ A-3).
#[derive(Args)]
pub struct ClaimsCmd {
    /// The text to annotate (reads from stdin if not provided).
    pub text: Option<String>,
}

impl ClaimsCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        let text = get_text(&self.text)?;
        let report = annotate_claims(&text).map_err(|e| anyhow::anyhow!(e))?;

        match format {
            "json" => println!("{}", serde_json::to_string_pretty(&report)?),
            "markdown" => println!("{}", to_markdown(&report)),
            _ => print_plain(&report),
        }
        Ok(())
    }
}

fn risk_color(risk: RiskLevel) -> colored::ColoredString {
    match risk {
        RiskLevel::Safe => "SAFE".green(),
        RiskLevel::Low => "LOW".yellow(),
        RiskLevel::Medium => "MEDIUM".yellow().bold(),
        RiskLevel::High => "HIGH".red().bold(),
        _ => "RISK".white(),
    }
}

fn print_plain(report: &ClaimAnnotatedReport) {
    println!("{}", "━".repeat(60).dimmed());
    println!(
        "{} {}",
        "Overall Risk:".bold(),
        risk_color(report.overall_risk)
    );
    println!(
        "{} {}  {} {}",
        "Safe claims:".bold(),
        report.safe_count.to_string().green(),
        "Risky:".bold(),
        report.risky_count.to_string().red()
    );
    println!(
        "{} {}  {} {}  {} {}  {} {}  {} {}",
        "Supported:".bold(),
        report.supported_count.to_string().green(),
        "Contradicted:".bold(),
        report.contradicted_count.to_string().red(),
        "Novel:".bold(),
        report.novel_count.to_string().yellow(),
        "Unresolved:".bold(),
        report.unresolved_count,
        "Missing context:".bold(),
        report.missing_context_count,
    );
    let contradiction_ready = report
        .claims
        .iter()
        .filter(|claim| annotation_to_triple(claim).supports_contradiction())
        .count();
    println!(
        "{} {}",
        "Contradiction-ready triples:".bold(),
        contradiction_ready
    );
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header([
        "ID",
        "Role",
        "Mode",
        "Evidence",
        "Triple conf",
        "Ready",
        "Risk",
        "Claim",
        "Issues",
    ]);

    for claim in &report.claims {
        let triple = annotation_to_triple(claim);
        let ready = if triple.supports_contradiction() {
            "yes".to_string()
        } else {
            "no".to_string()
        };
        let issues: Vec<&str> = claim
            .illusion_issues
            .iter()
            .chain(claim.antinomy_issues.iter())
            .chain(claim.paralogism_issues.iter())
            .map(String::as_str)
            .collect();

        let issues_str = if issues.is_empty() {
            "—".to_string()
        } else {
            issues.join("; ")
        };

        let text_preview: String = claim.text.chars().take(60).collect();
        let text_display = if claim.text.len() > 60 {
            format!("{}…", text_preview)
        } else {
            text_preview
        };

        table.add_row([
            &claim.claim_id,
            &claim.source_role.to_string(),
            &claim.modality.to_string(),
            &claim.evidence.status.to_string(),
            &format!("{:.2}", triple.extraction_confidence.value()),
            &ready,
            &format!("{}", claim.risk),
            &text_display,
            &issues_str,
        ]);
    }

    println!("{table}");
    println!("{}", "━".repeat(60).dimmed());
}

fn to_markdown(report: &ClaimAnnotatedReport) -> String {
    let mut md = String::new();
    md.push_str("# Per-Claim Epistemic Annotation\n\n");
    md.push_str(&format!("**Overall Risk:** {}\n", report.overall_risk));
    md.push_str(&format!(
        "**Safe claims:** {}  **Risky:** {}\n\n",
        report.safe_count, report.risky_count
    ));
    md.push_str(&format!(
        "**Supported:** {}  **Contradicted:** {}  **Novel:** {}  **Unresolved:** {}  **Missing context:** {}\n\n",
        report.supported_count,
        report.contradicted_count,
        report.novel_count,
        report.unresolved_count,
        report.missing_context_count,
    ));
    let contradiction_ready = report
        .claims
        .iter()
        .filter(|claim| annotation_to_triple(claim).supports_contradiction())
        .count();
    md.push_str(&format!(
        "**Contradiction-ready triples:** {}\n\n",
        contradiction_ready
    ));
    md.push_str(
        "| ID | Role | Modality | Evidence | Triple conf | Ready | Risk | Claim | Issues |\n",
    );
    md.push_str("|---|---|---|---|---|---|---|---|---|\n");

    for c in &report.claims {
        let triple = annotation_to_triple(c);
        let issues: Vec<&str> = c
            .illusion_issues
            .iter()
            .chain(c.antinomy_issues.iter())
            .chain(c.paralogism_issues.iter())
            .map(String::as_str)
            .collect();
        let issues_str = if issues.is_empty() {
            "—".to_string()
        } else {
            issues.join("; ")
        };
        md.push_str(&format!(
            "| {} | {} | {} | {} | {:.2} | {} | {} | {} | {} |\n",
            c.claim_id,
            c.source_role,
            c.modality,
            c.evidence.status,
            triple.extraction_confidence.value(),
            if triple.supports_contradiction() {
                "yes"
            } else {
                "no"
            },
            c.risk,
            c.text,
            issues_str
        ));
    }

    md
}
