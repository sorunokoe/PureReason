//! # `pure-reason train` command (TRIZ S-10)
//!
//! Reads feedback.jsonl, proposes WorldModel schema patches, and saves to
//! `~/.pure-reason/world_schema.toml`. The schema is loaded at pipeline startup
//! and used to initialise domain-specific category expectations.
//!
//! ## Usage
//! ```
//! # Propose patches (dry-run, do not save)
//! pure-reason train --dry-run
//!
//! # Apply patches and save schema
//! pure-reason train
//!
//! # Show current schema
//! pure-reason train --show
//! ```

use anyhow::Result;
use clap::Args;
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Table};
use serde_json;

use pure_reason_core::world_schema::SchemaLearner;

#[derive(Args)]
pub struct TrainArgs {
    /// Show current schema without proposing patches.
    #[arg(long)]
    pub show: bool,

    /// Propose patches but do not save them.
    #[arg(long)]
    pub dry_run: bool,

    /// Output format: plain | json
    #[arg(long, default_value = "plain")]
    pub format: String,
}

pub fn run(args: &TrainArgs) -> Result<()> {
    let learner = SchemaLearner::new();

    if args.show {
        let schema = learner.load_schema();
        match args.format.as_str() {
            "json" => println!("{}", serde_json::to_string_pretty(&schema)?),
            _ => {
                if schema.domains.is_empty() {
                    println!("No schema learned yet. Run with corrections first.");
                    return Ok(());
                }
                let mut table = Table::new();
                table.load_preset(UTF8_FULL);
                table.set_header(["Domain", "Categories", "Signals", "Training Count"]);
                for d in &schema.domains {
                    table.add_row([
                        d.name.as_str(),
                        &d.expected_categories.join(", "),
                        &d.causal_signals.join(", "),
                        &d.training_count.to_string(),
                    ]);
                }
                println!("{table}");
            }
        }
        return Ok(());
    }

    // Propose patches
    let (patches, new_schema) = learner.propose_patches();

    if patches.is_empty() {
        println!(
            "{} No schema patches proposed (no feedback events found).",
            "ℹ".cyan()
        );
        println!("  Record corrections with: pure-reason feedback log ...");
        return Ok(());
    }

    match args.format.as_str() {
        "json" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "patches": patches,
                    "schema": new_schema,
                }))?
            );
        }
        _ => {
            println!(
                "{} {} schema patch(es) proposed:",
                "→".green().bold(),
                patches.len()
            );
            println!();
            let mut table = Table::new();
            table.load_preset(UTF8_FULL);
            table.set_header(["Domain", "Category", "Signal", "Reason"]);
            for p in &patches {
                table.add_row([
                    p.domain.as_str(),
                    p.category.as_str(),
                    p.signal.as_deref().unwrap_or("—"),
                    p.reason.as_str(),
                ]);
            }
            println!("{table}");

            if args.dry_run {
                println!(
                    "\n{} Dry-run: schema NOT saved. Remove --dry-run to apply.",
                    "⚠".yellow()
                );
            } else {
                learner.save_schema(&new_schema)?;
                println!(
                    "\n{} Schema saved to: {}",
                    "✓".green().bold(),
                    learner.schema_path().display()
                );
            }
        }
    }

    Ok(())
}
