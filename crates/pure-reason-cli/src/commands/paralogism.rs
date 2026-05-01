use super::get_text;
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use pure_reason_core::{aesthetic::Intuition, dialectic::paralogisms::ParalogismDetector};

/// Detect paralogisms (invalid self-referential reasoning).
#[derive(Args)]
pub struct ParalogismCmd {
    pub text: Option<String>,
}

impl ParalogismCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        let text = get_text(&self.text)?;
        let intuition = Intuition::from_text(&text).map_err(|e| anyhow::anyhow!(e))?;
        let reports = ParalogismDetector::detect(&intuition.propositions());

        match format {
            "json" => println!("{}", serde_json::to_string_pretty(&reports)?),
            _ => {
                println!("{}", "Paralogism Detection".bold().underline());
                let any = reports.iter().any(|r| r.has_paralogisms);
                if !any {
                    println!("{} No paralogisms detected.", "✓".green());
                } else {
                    for r in reports.iter().filter(|r| r.has_paralogisms) {
                        println!("{} {}", "⚠".yellow(), r.critique);
                    }
                }
            }
        }
        Ok(())
    }
}
