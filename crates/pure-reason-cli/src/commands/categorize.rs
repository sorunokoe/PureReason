use super::get_text;
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Table};
use pure_reason_core::{aesthetic::Intuition, analytic::categories::CategoryAnalysis};

/// Apply the 12 Kantian categories to text.
#[derive(Args)]
pub struct CategorizeCmd {
    pub text: Option<String>,
}

impl CategorizeCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        let text = get_text(&self.text)?;
        let intuition = Intuition::from_text(&text).map_err(|e| anyhow::anyhow!(e))?;
        let propositions = intuition.propositions();
        let analysis = CategoryAnalysis::from_propositions(&propositions);

        match format {
            "json" => println!("{}", serde_json::to_string_pretty(&analysis)?),
            _ => {
                println!("{}", "Kantian Category Analysis".bold().underline());
                println!();
                let mut table = Table::new();
                table.load_preset(UTF8_FULL);
                table.set_header(["Group", "Category", "Confidence", "Evidence"]);
                for app in &analysis.applications {
                    if app.confidence.value() > 0.0 {
                        table.add_row([
                            &format!("{:?}", app.category.group()),
                            app.category.name(),
                            &format!("{:.3}", app.confidence.value()),
                            &app.evidence.join(", "),
                        ]);
                    }
                }
                println!("{table}");
                if let Some(dom) = analysis.dominant {
                    println!(
                        "\nDominant: {} — {}",
                        dom.name().cyan().bold(),
                        dom.description()
                    );
                }
            }
        }
        Ok(())
    }
}
