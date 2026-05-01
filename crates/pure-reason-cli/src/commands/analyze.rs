use super::{get_text, print_report};
use anyhow::Result;
use clap::Args;
use pure_reason_core::pipeline::KantianPipeline;

/// Run the full Kantian pipeline analysis on text.
#[derive(Args)]
pub struct AnalyzeCmd {
    /// The text to analyze (reads from stdin if not provided)
    pub text: Option<String>,
}

impl AnalyzeCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        let text = get_text(&self.text)?;
        let pipeline = KantianPipeline::new();
        let report = pipeline.process(&text).map_err(|e| anyhow::anyhow!(e))?;
        print_report(&report, format)
    }
}
