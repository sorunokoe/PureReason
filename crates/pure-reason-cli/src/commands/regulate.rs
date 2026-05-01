use super::get_text;
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Table};
use pure_reason_core::{
    dialectic::{OverreachKind, RegulativeTransformation},
    pipeline::{KantianPipeline, RiskLevel},
};

/// Convert constitutive epistemic overreach to regulative form.
///
/// This is the feature that changes the game: instead of flagging hallucinations
/// after the fact, it rewrites them into structurally sound regulative claims —
/// automatically, without an LLM, grounded in Kant's Transcendental Dialectic.
#[derive(Args)]
pub struct RegulateCmd {
    /// The text to regulate (reads from stdin if not provided)
    pub text: Option<String>,

    /// Show the full epistemic certificate for each transformation
    #[arg(long, short = 'c')]
    pub certificate: bool,

    /// Show only the regulated text (no transformation details)
    #[arg(long, short = 'q')]
    pub quiet: bool,
}

impl RegulateCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        let text = get_text(&self.text)?;
        let pipeline = KantianPipeline::new();
        let report = pipeline.process(&text).map_err(|e| anyhow::anyhow!(e))?;

        match format {
            "json" => {
                #[derive(serde::Serialize)]
                struct RegulateOutput<'a> {
                    original_text: &'a str,
                    regulated_text: &'a str,
                    transformation_count: usize,
                    transformations: &'a [pure_reason_core::dialectic::RegulativeTransformation],
                    risk: &'a str,
                }
                let out = RegulateOutput {
                    original_text: &report.input,
                    regulated_text: &report.regulated_text,
                    transformation_count: report.transformations.len(),
                    transformations: &report.transformations,
                    risk: &report.verdict.risk.to_string(),
                };
                println!("{}", serde_json::to_string_pretty(&out)?);
            }
            "markdown" => {
                println!("# Regulative Transformation Report\n");
                println!("**Risk Level:** {}", report.verdict.risk);
                println!("**Transformations:** {}\n", report.transformations.len());
                if report.transformations.is_empty() {
                    println!("✓ No constitutive overreach detected. Input is within the bounds of legitimate knowledge.\n");
                } else {
                    for (i, t) in report.transformations.iter().enumerate() {
                        println!("## Transformation {}", i + 1);
                        println!("- **Idea:** {}", t.transcendental_idea.name());
                        println!("- **Original:** {}", t.original);
                        println!("- **Regulated:** {}", t.regulated);
                        println!("- **Principle:** {}\n", t.regulative_principle);
                    }
                    println!("## Regulated Text\n\n> {}", report.regulated_text);
                }
            }
            _ => {
                if self.quiet {
                    println!("{}", report.regulated_text);
                } else {
                    print_regulate_plain(
                        &report.input,
                        &report.regulated_text,
                        &report.transformations,
                        &report.verdict.risk,
                        self.certificate,
                    );
                }
            }
        }

        Ok(())
    }
}

fn print_regulate_plain(
    original: &str,
    regulated_text: &str,
    transformations: &[RegulativeTransformation],
    risk: &RiskLevel,
    show_cert: bool,
) {
    let divider = "═".repeat(64);

    println!("{}", divider.dimmed());
    println!("{}", "  REGULATIVE TRANSFORMATION REPORT".bold().cyan());
    println!(
        "{}",
        "  PureReason — Kant's Critique of Pure Reason".dimmed()
    );
    println!("{}", divider.dimmed());
    println!();

    // Input summary
    println!("{} {}", "Risk:".bold(), format_risk(risk));
    println!(
        "{} {}",
        "Constitutive claims detected:".bold(),
        if transformations.is_empty() {
            "0 — input is within bounds of legitimate knowledge"
                .green()
                .to_string()
        } else {
            transformations.len().to_string().red().bold().to_string()
        }
    );
    println!();

    if transformations.is_empty() {
        println!("{} No constitutive overreach detected.", "✓".green().bold());
        println!(
            "  The input remains within the bounds of possible experience and legitimate knowledge."
        );
        println!();
        println!("{}", divider.dimmed());
        return;
    }

    // Per-transformation details
    for (i, t) in transformations.iter().enumerate() {
        let n = i + 1;
        println!(
            "{}",
            format!("  ── TRANSFORMATION {n} ──────────────────────────────────").bold()
        );
        println!();

        // Idea and overreach kind
        println!(
            "  {} {}",
            "Transcendental Idea:".bold(),
            t.transcendental_idea.name().cyan()
        );
        println!(
            "  {} {}",
            "Overreach Kind:".bold(),
            format_overreach_kind(&t.overreach_kind)
        );
        println!();

        // Original (constitutive)
        println!(
            "  {} {}",
            "ORIGINAL (Constitutive):".bold().red(),
            "⚠".red()
        );
        println!("  ┌{}", "─".repeat(60));
        for line in wrap_text(&t.original, 58) {
            println!("  │ {}", line.italic());
        }
        println!("  └{}", "─".repeat(60));
        println!();

        // Regulated form
        println!(
            "  {} {}",
            "REGULATED (Regulative):".bold().green(),
            "✓".green()
        );
        println!("  ┌{}", "─".repeat(60));
        for line in wrap_text(&t.regulated, 58) {
            println!("  │ {}", line);
        }
        println!("  └{}", "─".repeat(60));
        println!();

        // Regulative principle
        println!("  {}", "Regulative Principle:".bold().yellow());
        for line in wrap_text(&t.regulative_principle, 60) {
            println!("  • {}", line.dimmed());
        }
        println!();

        // Epistemic certificate (optional)
        if show_cert {
            println!("  {}", "Epistemic Certificate:".bold());
            let mut table = Table::new();
            table.load_preset(UTF8_FULL);
            table.set_header(["Field", "Value"]);
            table.add_row(["Original Use", "Constitutive ✗ (illegitimate)"]);
            table.add_row(["Regulated Use", "Regulative  ✓ (legitimate)"]);
            table.add_row([
                "Kantian Principle",
                &truncate(&t.certificate.kantian_principle, 60),
            ]);
            table.add_row([
                "Resolution",
                &truncate(&t.certificate.kantian_resolution, 60),
            ]);
            println!("{table}");
            println!();
        }
    }

    // Regulated text block
    println!("{}", divider.dimmed());
    println!("{}", "  REGULATED TEXT".bold().green());
    println!("{}", divider.dimmed());
    println!();
    for line in wrap_text(regulated_text, 70) {
        println!("  {}", line);
    }
    println!();

    // If the regulated text differs, show the diff summary
    if original != regulated_text {
        println!("{}", divider.dimmed());
        println!("{}", "  ORIGINAL TEXT (for reference)".bold().dimmed());
        println!("{}", divider.dimmed());
        println!();
        for line in wrap_text(original, 70) {
            println!("  {}", line.dimmed().italic());
        }
        println!();
    }

    println!("{}", divider.dimmed());
    println!(
        "  {} {} constitutive claim(s) corrected. Output is epistemically certified.",
        "✓".green().bold(),
        transformations.len()
    );
    println!("{}", divider.dimmed());
}

// ─── Formatting helpers ───────────────────────────────────────────────────────

fn format_risk(risk: &RiskLevel) -> colored::ColoredString {
    match risk {
        RiskLevel::Safe => "SAFE".green().bold(),
        RiskLevel::Low => "LOW".yellow(),
        RiskLevel::Medium => "MEDIUM".yellow().bold(),
        RiskLevel::High => "HIGH".red().bold(),
        _ => "RISK".white(),
    }
}

fn format_overreach_kind(kind: &OverreachKind) -> String {
    match kind {
        OverreachKind::SoulParalogism(k) => format!("Soul/Self Paralogism ({k:?})"),
        OverreachKind::WorldAntinomy(id) => format!("World Antinomy ({id:?})"),
        OverreachKind::GodIdeal => "Theological Ideal Hypostatization".to_string(),
        OverreachKind::EpistemicCertainty => "Epistemic Overreach / Certainty Claim".to_string(),
    }
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();

    for word in words {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current.clone());
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
