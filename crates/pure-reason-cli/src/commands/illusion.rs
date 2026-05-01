use super::get_text;
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use pure_reason_core::{aesthetic::Intuition, dialectic::IllusionDetector};

/// Detect transcendental illusions (epistemic overreach / hallucination analogues).
#[derive(Args)]
pub struct IllusionCmd {
    pub text: Option<String>,
}

impl IllusionCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        let text = get_text(&self.text)?;
        let intuition = Intuition::from_text(&text).map_err(|e| anyhow::anyhow!(e))?;
        let illusions = IllusionDetector::detect(&intuition.propositions());

        match format {
            "json" => println!("{}", serde_json::to_string_pretty(&illusions)?),
            _ => {
                println!("{}", "Transcendental Illusion Detection".bold().underline());
                if illusions.is_empty() {
                    println!("{} No transcendental illusions detected.", "✓".green());
                } else {
                    for ill in &illusions {
                        let sev = format!("{:?}", ill.severity);
                        println!("{} [{}] {}", "⚠".yellow(), sev.yellow(), ill.description);
                        println!("  Idea: {}  |  Kind: {:?}", ill.idea.name(), ill.kind);
                    }
                }
            }
        }
        Ok(())
    }
}
