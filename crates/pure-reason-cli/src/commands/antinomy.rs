use super::get_text;
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use pure_reason_core::{aesthetic::Intuition, dialectic::antinomies::AntinomyDetector};

/// Scan for antinomies (contradictions) in text.
#[derive(Args)]
pub struct AntinomyCmd {
    pub text: Option<String>,
}

impl AntinomyCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        let text = get_text(&self.text)?;
        let intuition = Intuition::from_text(&text).map_err(|e| anyhow::anyhow!(e))?;
        let reports = AntinomyDetector::detect(&intuition.propositions());

        match format {
            "json" => println!("{}", serde_json::to_string_pretty(&reports)?),
            _ => {
                println!("{}", "Antinomy Scan".bold().underline());
                if reports.is_empty() {
                    println!("{} No antinomies detected.", "✓".green());
                } else {
                    for r in &reports {
                        let icon = if r.has_conflict {
                            "✗".red().to_string()
                        } else {
                            "~".yellow().to_string()
                        };
                        println!("{} {:?}: {}", icon, r.antinomy, r.description);
                        println!("  Resolution: {}", r.resolution);
                    }
                }
            }
        }
        Ok(())
    }
}
