//! # SLA Command
//!
//! Produces an SLA-style epistemic health report by running the local Kantian
//! pipeline on a sample of text.  No HTTP dependency required.
//!
//! ```
//! pure-reason sla "The algorithm guarantees 100% accuracy."
//! pure-reason sla --api-url http://127.0.0.1:8080   # shows note about local mode
//! ```

use super::get_text;
use anyhow::Result;
use chrono::Utc;
use clap::Args;
use colored::Colorize;
use pure_reason_core::pipeline::{KantianPipeline, RiskLevel};

#[derive(Args)]
pub struct SlaCmd {
    /// Text to analyse for SLA report (reads from stdin if omitted)
    pub text: Option<String>,

    /// API URL (informational only; report is computed locally)
    #[arg(long, default_value = "http://127.0.0.1:8080")]
    pub api_url: String,
}

impl SlaCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        let text = get_text(&self.text)?;
        let pipeline = KantianPipeline::new();
        let report = pipeline.process(&text).map_err(|e| anyhow::anyhow!(e))?;

        let risk_str = report.verdict.risk.to_string();
        let illusion_count = report.dialectic.illusions.len();
        let antinomy_count = report
            .dialectic
            .antinomies
            .iter()
            .filter(|a| a.has_conflict)
            .count();
        let paralogism_count = report
            .dialectic
            .paralogisms
            .iter()
            .map(|p| p.detected.len())
            .sum::<usize>();
        let transformation_count = report.transformations.len();
        let auto_regulated = transformation_count > 0;

        let health_score = compute_health(
            &report.verdict.risk,
            illusion_count,
            antinomy_count,
            paralogism_count,
        );
        let sla_status = if health_score >= 95 {
            "MEETING SLA"
        } else if health_score >= 80 {
            "AT RISK"
        } else {
            "BREACHING SLA"
        };

        let generated_at = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        match format {
            "json" => {
                let out = serde_json::json!({
                    "generated_at": generated_at,
                    "api_url": self.api_url,
                    "mode": "local",
                    "sla_status": sla_status,
                    "health_score": health_score,
                    "risk_level": risk_str,
                    "illusion_count": illusion_count,
                    "antinomy_count": antinomy_count,
                    "paralogism_count": paralogism_count,
                    "transformation_count": transformation_count,
                    "auto_regulated": auto_regulated,
                    "summary": report.summary,
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            }
            "markdown" => {
                println!("# PureReason SLA Report\n");
                println!("| Field | Value |");
                println!("|-------|-------|");
                println!("| Generated | {} |", generated_at);
                println!("| Mode | local (no HTTP) |");
                println!("| SLA Status | **{}** |", sla_status);
                println!("| Health Score | {}% |", health_score);
                println!("| Risk Level | {} |", risk_str);
                println!("| Illusions | {} |", illusion_count);
                println!("| Antinomies | {} |", antinomy_count);
                println!("| Paralogisms | {} |", paralogism_count);
                println!("| Auto-Regulated | {} |", auto_regulated);
                println!("\n**Summary:** {}", report.summary);
            }
            _ => print_plain(
                sla_status,
                health_score,
                &risk_str,
                illusion_count,
                antinomy_count,
                paralogism_count,
                auto_regulated,
                &report.summary,
                &generated_at,
                &self.api_url,
            ),
        }

        Ok(())
    }
}

fn compute_health(
    risk: &RiskLevel,
    illusions: usize,
    antinomies: usize,
    paralogisms: usize,
) -> u32 {
    let base: f64 = match risk {
        RiskLevel::Safe => 100.0,
        RiskLevel::Low => 90.0,
        RiskLevel::Medium => 70.0,
        RiskLevel::High => 40.0,
        _ => 50.0,
    };
    let penalty =
        (illusions as f64 * 5.0 + antinomies as f64 * 8.0 + paralogisms as f64 * 4.0).min(base);
    (base - penalty).clamp(0.0, 100.0) as u32
}

// This function is a plain-text printer that needs all its display parameters as separate
// arguments. Grouping them into a struct would obscure the display logic without real benefit.
#[allow(clippy::too_many_arguments)]
fn print_plain(
    sla_status: &str,
    health_score: u32,
    risk_str: &str,
    illusion_count: usize,
    antinomy_count: usize,
    paralogism_count: usize,
    auto_regulated: bool,
    summary: &str,
    generated_at: &str,
    api_url: &str,
) {
    let divider = "═".repeat(60);
    println!("{}", divider.dimmed());
    println!("{}", "  PURЕРEASON SLA REPORT".bold().cyan());
    println!("{}", format!("  {} (local mode)", api_url).dimmed());
    println!("{}", divider.dimmed());

    let status_colored = match sla_status {
        "MEETING SLA" => sla_status.green().bold().to_string(),
        "AT RISK" => sla_status.yellow().bold().to_string(),
        _ => sla_status.red().bold().to_string(),
    };
    let health_colored = if health_score >= 95 {
        format!("{}%", health_score).green().bold().to_string()
    } else if health_score >= 80 {
        format!("{}%", health_score).yellow().bold().to_string()
    } else {
        format!("{}%", health_score).red().bold().to_string()
    };

    println!("  {} {}", "SLA Status:".bold(), status_colored);
    println!("  {} {}", "Health Score:".bold(), health_colored);
    println!("  {} {}", "Risk Level:".bold(), risk_str);
    println!("  {} {}", "Generated:".bold(), generated_at);
    println!();
    println!(
        "  {} {} illusion(s)  |  {} antinomy(s)  |  {} paralogism(s)",
        "Issues:".bold(),
        illusion_count,
        antinomy_count,
        paralogism_count
    );
    println!(
        "  {} {}",
        "Auto-Regulated:".bold(),
        if auto_regulated {
            "yes".green().to_string()
        } else {
            "no".dimmed().to_string()
        }
    );
    println!();
    println!("  {} {}", "Summary:".bold(), summary.dimmed());
    println!("{}", divider.dimmed());
}
