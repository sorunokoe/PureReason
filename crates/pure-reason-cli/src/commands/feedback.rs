use anyhow::Result;
use clap::{Args, Subcommand};
use colored::Colorize;
use pure_reason_core::feedback::{FeedbackCollector, FeedbackEvent, FeedbackKind, TrainingAction};

/// FeedbackCollector commands (TRIZ A-2): log corrections and review training suggestions.
#[derive(Args)]
pub struct FeedbackCmd {
    #[command(subcommand)]
    pub action: FeedbackAction,
}

#[derive(Subcommand)]
pub enum FeedbackAction {
    /// Record a missed illusion detection.
    Missed {
        /// The input text that should have been flagged.
        text: String,
        /// The illusion kind (e.g. Theological, Cosmological, Psychological).
        #[clap(long)]
        kind: String,
        /// The phrase that should have triggered detection.
        #[clap(long)]
        phrase: String,
        /// Optional notes.
        #[clap(long)]
        notes: Option<String>,
    },
    /// Record a false positive (incorrect detection).
    FalsePositive {
        /// The input text that was incorrectly flagged.
        text: String,
        /// The illusion kind that was wrongly detected.
        #[clap(long)]
        kind: String,
        /// The phrase that triggered the false positive.
        #[clap(long)]
        phrase: String,
    },
    /// Record wrong risk level.
    WrongRisk {
        /// The input text.
        text: String,
        /// Expected risk level (SAFE/LOW/MEDIUM/HIGH).
        #[clap(long)]
        expected: String,
        /// Actual risk level returned.
        #[clap(long)]
        got: String,
    },
    /// Show training suggestions derived from feedback history.
    Train,
    /// Show feedback store statistics.
    Stats,
}

impl FeedbackCmd {
    pub async fn run(&self, _format: &str) -> Result<()> {
        let collector = FeedbackCollector::new();

        match &self.action {
            FeedbackAction::Missed {
                text,
                kind,
                phrase,
                notes,
            } => {
                let mut event = FeedbackEvent::new(
                    text,
                    FeedbackKind::MissedIllusion {
                        kind: kind.clone(),
                        phrase: phrase.clone(),
                    },
                );
                if let Some(n) = notes {
                    event = event.with_notes(n);
                }
                collector.record(&event).map_err(|e| anyhow::anyhow!(e))?;
                println!(
                    "{} Feedback recorded to {}",
                    "✓".green(),
                    collector.path().display()
                );
            }
            FeedbackAction::FalsePositive { text, kind, phrase } => {
                let event = FeedbackEvent::new(
                    text,
                    FeedbackKind::FalsePositive {
                        kind: kind.clone(),
                        phrase: phrase.clone(),
                    },
                );
                collector.record(&event).map_err(|e| anyhow::anyhow!(e))?;
                println!(
                    "{} Feedback recorded to {}",
                    "✓".green(),
                    collector.path().display()
                );
            }
            FeedbackAction::WrongRisk {
                text,
                expected,
                got,
            } => {
                let event = FeedbackEvent::new(
                    text,
                    FeedbackKind::WrongRiskLevel {
                        expected: expected.clone(),
                        got: got.clone(),
                    },
                );
                collector.record(&event).map_err(|e| anyhow::anyhow!(e))?;
                println!(
                    "{} Feedback recorded to {}",
                    "✓".green(),
                    collector.path().display()
                );
            }
            FeedbackAction::Train => {
                let suggestions = collector
                    .suggest_training()
                    .map_err(|e| anyhow::anyhow!(e))?;
                if suggestions.is_empty() {
                    println!("{} No training suggestions yet. Record ≥2 similar issues to get suggestions.",
                        "ℹ".cyan());
                    return Ok(());
                }
                println!("{}", "Training Suggestions".bold().underline());
                println!();
                for s in &suggestions {
                    let action = match s.action {
                        TrainingAction::AddSignal => "ADD signal phrase".green(),
                        TrainingAction::RemoveSignal => {
                            "REMOVE signal phrase (false positive)".red()
                        }
                    };
                    println!("  {} [{} occurrences]", action, s.occurrences);
                    println!("    Kind:   {}", s.kind.cyan());
                    println!("    Phrase: \"{}\"", s.phrase.yellow());
                    println!();
                }
            }
            FeedbackAction::Stats => {
                let events = collector.load_all().map_err(|e| anyhow::anyhow!(e))?;
                let missed = events
                    .iter()
                    .filter(|e| matches!(&e.correction, FeedbackKind::MissedIllusion { .. }))
                    .count();
                let false_pos = events
                    .iter()
                    .filter(|e| matches!(&e.correction, FeedbackKind::FalsePositive { .. }))
                    .count();
                let wrong_risk = events
                    .iter()
                    .filter(|e| matches!(&e.correction, FeedbackKind::WrongRiskLevel { .. }))
                    .count();

                println!("{}", "Feedback Statistics".bold().underline());
                println!("  Store:        {}", collector.path().display());
                println!("  Total events: {}", events.len());
                println!("  Missed:       {}", missed);
                println!("  False pos:    {}", false_pos);
                println!("  Wrong risk:   {}", wrong_risk);
            }
        }
        Ok(())
    }
}
