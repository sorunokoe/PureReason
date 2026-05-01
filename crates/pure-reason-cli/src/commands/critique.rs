use super::{get_text, print_report};
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use pure_reason_core::pipeline::KantianPipeline;

/// Full Kantian critique of text or LLM output.
///
/// Runs the complete pipeline and prints an exhaustive critique
/// covering all faculties: Aesthetic, Analytic, Dialectic, Methodology, Wittgenstein.
#[derive(Args)]
pub struct CritiqueCmd {
    pub text: Option<String>,
}

impl CritiqueCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        let text = get_text(&self.text)?;
        let pipeline = KantianPipeline::new();
        let report = pipeline.process(&text).map_err(|e| anyhow::anyhow!(e))?;

        match format {
            "markdown" => println!("{}", report.to_markdown()),
            "json" => println!("{}", report.to_json().map_err(|e| anyhow::anyhow!(e))?),
            _ => {
                println!("{}", "═".repeat(70).cyan());
                println!("{}", "  KANTIAN CRITIQUE".bold().cyan());
                println!(
                    "{}",
                    "  Critique of Pure Reason — Full Faculty Analysis".dimmed()
                );
                println!("{}", "═".repeat(70).cyan());
                println!();

                print_report(&report, "plain")?;

                // Extended critique
                println!();
                println!(
                    "{}",
                    "Methodology — Canon (Legitimate Uses):".bold().underline()
                );
                for use_ in &report.methodology.legitimate_uses {
                    println!("  {} {:?}: {}", "✓".green(), use_.kind, use_.description);
                }
                if report.methodology.legitimate_uses.is_empty() {
                    println!("  No explicitly legitimate uses identified.");
                }

                println!();
                println!(
                    "{}",
                    "Methodology — Discipline (Violations):".bold().underline()
                );
                if report.methodology.discipline_violations.is_empty() {
                    println!("  {} No disciplinary violations.", "✓".green());
                } else {
                    for v in &report.methodology.discipline_violations {
                        println!("  {} {}", "✗".red(), v.correction);
                    }
                }

                println!();
                println!("{}", "Wittgenstein — Language Games:".bold().underline());
                println!(
                    "  {}",
                    report.wittgenstein.game_analysis.interpretation_note
                );

                println!();
                println!(
                    "{}",
                    "Wittgenstein — Speakable / Showable:".bold().underline()
                );
                println!(
                    "  Speakable: {} proposition(s)",
                    report.wittgenstein.speakable_boundary.speakable.len()
                );
                if !report
                    .wittgenstein
                    .speakable_boundary
                    .showable_only
                    .is_empty()
                {
                    println!(
                        "  Show-only: {} proposition(s) — these cannot be said, only shown",
                        report.wittgenstein.speakable_boundary.showable_only.len()
                    );
                }

                println!();
                println!("{}", "Architectonic:".bold().underline());
                let arch = report.methodology.system_description;
                println!("  {}", arch.title.cyan());
                println!("  Unifying idea: {}", arch.unifying_idea.dimmed());
            }
        }
        Ok(())
    }
}
