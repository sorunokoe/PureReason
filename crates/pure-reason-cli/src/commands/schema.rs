use super::get_text;
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Table};
use pure_reason_core::{
    aesthetic::Intuition, analytic::categories::CategoryAnalysis, analytic::schematism::Schematism,
};

/// Generate schematized (temporal) context for a domain.
#[derive(Args)]
pub struct SchemaCmd {
    /// Domain or text to schematize
    pub text: Option<String>,
}

impl SchemaCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        let text = get_text(&self.text)?;
        let intuition = Intuition::from_text(&text).map_err(|e| anyhow::anyhow!(e))?;
        let propositions = intuition.propositions();
        let analysis = CategoryAnalysis::from_propositions(&propositions);
        let schematism = Schematism::new();
        let schemas = schematism.apply_to_analysis(&analysis, &intuition.time);

        match format {
            "json" => println!("{}", serde_json::to_string_pretty(&schemas)?),
            _ => {
                println!(
                    "{}",
                    "Schematism — Temporal Determinations".bold().underline()
                );
                println!();
                let mut table = Table::new();
                table.load_preset(UTF8_FULL);
                table.set_header([
                    "Category",
                    "Temporal Determination",
                    "Condition Met",
                    "Description",
                ]);
                for schema in &schemas {
                    table.add_row([
                        schema.category.name(),
                        &format!("{:?}", schema.determination),
                        if schema.instantiation.condition_met {
                            "✓"
                        } else {
                            "✗"
                        },
                        &schema.instantiation.description,
                    ]);
                }
                println!("{table}");
            }
        }
        Ok(())
    }
}
