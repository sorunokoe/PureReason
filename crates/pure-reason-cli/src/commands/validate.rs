use super::get_text;
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use pure_reason_core::{
    aesthetic::Intuition,
    dialectic::{DialecticLayer, DialecticReport},
    types::Faculty,
};

/// Run dialectical validation (illusions, antinomies, paralogisms).
#[derive(Args)]
pub struct ValidateCmd {
    pub text: Option<String>,
}

impl ValidateCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        let text = get_text(&self.text)?;
        let intuition = Intuition::from_text(&text).map_err(|e| anyhow::anyhow!(e))?;
        let propositions = intuition.propositions();
        let layer = DialecticLayer;
        let report: DialecticReport = layer.apply(propositions).map_err(|e| anyhow::anyhow!(e))?;

        match format {
            "json" => println!("{}", serde_json::to_string_pretty(&report)?),
            _ => {
                println!("{}", "Dialectical Validation Report".bold().underline());
                println!();
                println!("{}", report.summary);
                println!();

                if !report.illusions.is_empty() {
                    println!("{}", "Transcendental Illusions:".yellow().bold());
                    for ill in &report.illusions {
                        println!(
                            "  ⚠ [{}] {}",
                            format!("{:?}", ill.severity).yellow(),
                            ill.description
                        );
                    }
                    println!();
                }

                if report.antinomies.iter().any(|a| a.has_conflict) {
                    println!("{}", "Antinomies (Contradictions):".red().bold());
                    for anti in report.antinomies.iter().filter(|a| a.has_conflict) {
                        println!("  ✗ {:?}", anti.antinomy);
                        println!("    Resolution: {}", anti.resolution);
                    }
                    println!();
                }

                if report.paralogisms.iter().any(|p| p.has_paralogisms) {
                    println!("{}", "Paralogisms:".yellow().bold());
                    for par in report.paralogisms.iter().filter(|p| p.has_paralogisms) {
                        println!("  ⚠ {}", par.critique);
                    }
                }

                if !report.has_critical_illusions && report.illusions.is_empty() {
                    println!("{} Within bounds of legitimate knowledge.", "✓".green());
                }
            }
        }
        Ok(())
    }
}
