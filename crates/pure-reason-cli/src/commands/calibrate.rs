//! # calibrate — the primary ECS command
//!
//! The user-facing entry point for the PureReason Calibration Layer.
//!
//! Usage:
//! ```
//! pure-reason calibrate "The patient must have cancer."
//! pure-reason calibrate "The patient must have cancer." --domain medical
//! pure-reason calibrate "Findings are consistent with malignancy." --format json
//! ```
//!
//! Output: ECS score (0–100), band, epistemic mode, flags, and safe rewrite.

use super::{format_ecs, get_text};
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use pure_reason_core::{
    calibration::PipelineCalibration, pipeline::KantianPipeline, rewriter::RewriteDomain,
};

/// Compute the Epistemic Confidence Score (ECS) for text.
///
/// ECS (0–100) measures how well-calibrated an LLM output is:
/// whether its stated confidence matches its evidential warrant.
///
/// - 80–100 HIGH:     Epistemically calibrated. Safe for regulated use.
/// - 40–79  MODERATE: Some risk. Review for high-stakes decisions.
/// - 0–39   LOW:      Overconfident, contradictory, or overreaching.
#[derive(Args)]
pub struct CalibrateCmd {
    /// The text to calibrate (reads from stdin if not provided)
    pub text: Option<String>,

    /// Show the score breakdown (per-component contributions)
    #[arg(long)]
    pub breakdown: bool,

    /// Show the safe (regulative) rewrite if ECS < 80
    #[arg(long, default_value = "true")]
    pub rewrite: bool,

    /// Apply domain-specific regulative rewrite rules.
    /// Options: medical, legal, financial, technical, general (default)
    #[arg(long, default_value = "general")]
    pub domain: String,
}

impl CalibrateCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        let text = get_text(&self.text)?;
        let pipeline = KantianPipeline::new();
        let report = pipeline.process(&text).map_err(|e| anyhow::anyhow!(e))?;
        let cal = report.calibration();
        let domain = RewriteDomain::parse_domain(&self.domain);

        // Compute domain rewrite if requested and ECS is not HIGH
        let domain_rewrite = if !cal.calibrated && self.rewrite {
            let r = cal.rewrite_for_domain(domain);
            if r.changed {
                Some(r)
            } else {
                None
            }
        } else {
            None
        };

        match format {
            "json" => {
                let mut v = serde_json::to_value(&cal)?;
                if let Some(dr) = &domain_rewrite {
                    v["domain_rewrite"] = serde_json::to_value(dr)?;
                }
                println!("{}", serde_json::to_string_pretty(&v)?);
            }
            "markdown" => {
                println!("# Epistemic Calibration Report\n");
                println!("**ECS:** {}/100 — **{}**\n", cal.ecs, cal.band.label());
                println!("**Epistemic Mode:** {}\n", cal.epistemic_mode);
                println!("{}\n", cal.band.description());
                if !cal.flags.is_empty() {
                    println!("## Flags\n");
                    for flag in &cal.flags {
                        println!("- {}\n", flag);
                    }
                }
                if !cal.calibrated && self.rewrite {
                    println!("## Safe Version (General)\n");
                    println!("> {}\n", cal.safe_version);
                }
                if let Some(dr) = &domain_rewrite {
                    println!("## Domain Rewrite ({:?})\n", dr.domain);
                    println!("> {}\n", dr.regulated);
                    if !dr.rules_applied.is_empty() {
                        println!("**Rules applied:** {}\n", dr.rules_applied.len());
                    }
                }
                if self.breakdown {
                    let bd = &cal.score_breakdown;
                    println!("## Score Breakdown\n");
                    println!("| Component     | Score  | Weight |");
                    println!("|---------------|--------|--------|");
                    println!("| Modality      | {:.2}   | 35%    |", bd.modality);
                    println!("| Illusion      | {:.2}   | 30%    |", bd.illusion);
                    println!("| Antinomy      | {:.2}   | 20%    |", bd.antinomy);
                    println!("| Paralogism    | {:.2}   | 10%    |", bd.paralogism);
                    println!("| Game Stability| {:.2}   | 5%     |", bd.game_stability);
                }
            }
            _ => {
                // Plain output — the primary UX
                let separator = "━".repeat(60).dimmed();
                println!("{}", separator);

                // ECS banner
                println!(
                    "{} {}  {}",
                    "ECS:".bold(),
                    format_ecs(cal.ecs),
                    cal.band.description().dimmed(),
                );
                println!("{} {}", "Band:".bold(), format_band(cal.ecs));
                println!("{} {}", "Epistemic Mode:".bold(), cal.epistemic_mode.cyan());
                if !matches!(domain, RewriteDomain::General) {
                    println!("{} {:?}", "Domain:".bold(), domain);
                }
                println!();

                // Input preview
                let preview = text.chars().take(100).collect::<String>();
                let ellipsis = if text.len() > 100 { "…" } else { "" };
                println!(
                    "{} {}{}",
                    "Input:".bold().dimmed(),
                    preview.dimmed(),
                    ellipsis.dimmed()
                );
                println!();

                // Flags
                if cal.flags.is_empty() {
                    println!("  {} No epistemic issues detected.", "✓".green());
                } else {
                    println!("{}", "Flags:".bold().underline());
                    for flag in &cal.flags {
                        println!("  {} {}", "→".red(), flag);
                    }
                }

                // General safe rewrite
                if !cal.calibrated && self.rewrite && cal.safe_version != text {
                    println!();
                    println!("{}", "Safe Rewrite (General):".bold().underline());
                    println!("  {}", cal.safe_version.italic());
                }

                // Domain-specific rewrite
                if let Some(dr) = &domain_rewrite {
                    println!();
                    println!("{} {:?})", "Domain Rewrite (".bold().underline(), dr.domain);
                    println!("  {}", dr.regulated.italic().cyan());
                    if !dr.rules_applied.is_empty() {
                        println!("  {} rule(s) applied", dr.rules_applied.len());
                    }
                }

                // Optional breakdown
                if self.breakdown {
                    println!();
                    println!("{}", "Score Breakdown:".bold().underline());
                    let bd = &cal.score_breakdown;
                    println!("  Modality       {:.2}  (35%)", bd.modality);
                    println!("  Illusion       {:.2}  (30%)", bd.illusion);
                    println!("  Antinomy       {:.2}  (20%)", bd.antinomy);
                    println!("  Paralogism     {:.2}  (10%)", bd.paralogism);
                    println!("  Game Stability {:.2}   (5%)", bd.game_stability);
                }

                println!("{}", separator);
            }
        }

        Ok(())
    }
}

fn format_band(ecs: u8) -> colored::ColoredString {
    match ecs {
        80..=100 => "✅ HIGH — epistemically calibrated".green().bold(),
        40..=79 => "⚠️  MODERATE — human review recommended".yellow().bold(),
        _ => "🚫 LOW — flag for rewrite or abstention".red().bold(),
    }
}
