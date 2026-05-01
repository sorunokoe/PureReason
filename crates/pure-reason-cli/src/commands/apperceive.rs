//! # `pure-reason apperceive` command
//!
//! Processes text through the Kantian pipeline AND the Transcendental World Model.
//! Shows the WorldModel state (objects, causal rules, unity violations) after each input.
//!
//! ## Usage
//! ```
//! # Single text input
//! pure-reason apperceive "Heat causes expansion."
//!
//! # Interactive (multi-turn) mode
//! pure-reason apperceive --interactive
//!
//! # JSON output
//! pure-reason apperceive "Rain causes flooding." --format json
//! ```

use anyhow::Result;
use clap::Args;
use comfy_table::{presets::UTF8_FULL, Table};
use serde_json;

use pure_reason_core::{pipeline::KantianPipeline, unity::UnityChecker, world_model::WorldModel};

#[derive(Args)]
pub struct ApperceiveArgs {
    /// Text to process (omit for interactive multi-turn mode).
    pub text: Option<String>,

    /// Run in interactive multi-turn mode (read lines from stdin).
    #[arg(long, short = 'i')]
    pub interactive: bool,

    /// Output format: plain | json
    #[arg(long, default_value = "plain")]
    pub format: String,
}

pub fn run(args: &ApperceiveArgs) -> Result<()> {
    let mut pipeline = KantianPipeline::new();
    let mut world = WorldModel::new();
    let checker = UnityChecker::new();

    if args.interactive || args.text.is_none() {
        // Multi-turn interactive mode
        eprintln!("Kantian Apperception Engine — interactive mode");
        eprintln!("Type each turn and press Enter. Ctrl+D to finish.\n");

        let stdin = std::io::stdin();
        let mut line = String::new();
        loop {
            line.clear();
            match stdin.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {}
                Err(e) => return Err(e.into()),
            }
            let text = line.trim();
            if text.is_empty() {
                continue;
            }

            process_turn(text, &mut pipeline, &mut world, &checker, &args.format)?;
        }
    } else {
        let text = args.text.as_deref().unwrap();
        process_turn(text, &mut pipeline, &mut world, &checker, &args.format)?;
    }

    Ok(())
}

fn process_turn(
    text: &str,
    pipeline: &mut KantianPipeline,
    world: &mut WorldModel,
    checker: &UnityChecker,
    format: &str,
) -> Result<()> {
    let report = pipeline
        .process(text)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let violations = checker.update_and_check(world, &report);
    let predictions = world.predict_next();

    match format {
        "json" => {
            let out = serde_json::json!({
                "input": text,
                "world_model": {
                    "time_step": world.time_step,
                    "object_count": world.objects.len(),
                    "rule_count": world.rules.len(),
                    "objects": world.objects.keys().map(|k| k.0.clone()).collect::<Vec<_>>(),
                    "rules": world.rules.iter().map(|r| {
                        serde_json::json!({
                            "antecedent": r.antecedent,
                            "consequent": r.consequent,
                            "confidence": r.confidence,
                        })
                    }).collect::<Vec<_>>(),
                },
                "unity_violations": violations.len(),
                "violations": violations.iter().map(|v| serde_json::json!({
                    "kind": format!("{:?}", v.kind),
                    "description": v.description,
                })).collect::<Vec<_>>(),
                "predictions": predictions,
                "pipeline_verdict": format!("{:?}", report.verdict),
            });
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        _ => {
            println!("\n{}", world.summary());

            // Objects table
            if !world.objects.is_empty() {
                let mut table = Table::new();
                table.load_preset(UTF8_FULL);
                table.set_header(["Object", "Facts", "Latest Category"]);
                for (id, obj) in &world.objects {
                    let latest_cat = obj
                        .facts
                        .last()
                        .map(|f| format!("{:?}", f.category))
                        .unwrap_or_else(|| "—".to_string());
                    table.add_row([id.0.as_str(), &obj.facts.len().to_string(), &latest_cat]);
                }
                println!("{table}");
            }

            // Causal rules
            if !world.rules.is_empty() {
                println!("\nCausal Rules:");
                for r in &world.rules {
                    println!(
                        "  [{:.0}%] \"{}\" → \"{}\"",
                        r.confidence * 100.0,
                        r.antecedent,
                        r.consequent
                    );
                }
            }

            // Unity violations
            if violations.is_empty() {
                println!("\n✅ Unity: all Kantian unity conditions satisfied");
            } else {
                println!("\n⚠️  Unity Violations ({}):", violations.len());
                for v in &violations {
                    println!("  [{:?}] {}", v.kind, v.description);
                }
            }

            // Predictions
            if !predictions.is_empty() {
                println!("\nPredictions (from WorldModel):");
                for p in &predictions {
                    println!("  → {p}");
                }
            }

            // Pipeline verdict
            println!("\nPipeline verdict: {:?}", report.verdict);
        }
    }

    Ok(())
}
