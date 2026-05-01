use super::get_text;
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use pure_reason_core::wittgenstein::language_games::GameDetector;

/// Detect language game and form of life.
#[derive(Args)]
pub struct GameCmd {
    pub text: Option<String>,
}

impl GameCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        let text = get_text(&self.text)?;
        let detector = GameDetector::new();
        let analysis = detector.analyze(&text);

        match format {
            "json" => println!("{}", serde_json::to_string_pretty(&analysis)?),
            _ => {
                println!(
                    "{}",
                    "Language Game Detection (Wittgenstein)".bold().underline()
                );
                println!();
                if let Some(primary) = &analysis.primary_game {
                    println!(
                        "Primary game: {} (confidence: {:.2})",
                        primary.name.cyan().bold(),
                        primary.confidence
                    );
                    println!();
                    println!("Rules:");
                    for rule in &primary.rules {
                        println!("  • {}", rule);
                    }
                } else {
                    println!("No specific language game detected.");
                }
                if analysis.is_mixed {
                    println!();
                    println!(
                        "{} {}",
                        "⚠ Mixed games:".yellow(),
                        analysis.interpretation_note
                    );
                }
                println!();
                println!("{}", analysis.interpretation_note.dimmed());
            }
        }
        Ok(())
    }
}
