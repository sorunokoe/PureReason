use super::print_report;
use anyhow::Result;
use clap::Args;
use pure_reason_core::pipeline::KantianPipeline;
use std::io::{self, BufRead};

/// Read from stdin and output analysis to stdout (pipeline mode).
///
/// Reads one line at a time or the full stdin, then runs the Kantian pipeline.
/// Suitable for piping: `echo "text" | pure-reason pipeline`
#[derive(Args)]
pub struct PipelineCmd {
    /// Process each line independently (default: process all stdin as one block)
    #[arg(short, long)]
    pub line_by_line: bool,
}

impl PipelineCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        let pipeline = KantianPipeline::new();

        if self.line_by_line {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                let text = line?;
                if text.trim().is_empty() {
                    continue;
                }
                let report = pipeline.process(&text).map_err(|e| anyhow::anyhow!(e))?;
                print_report(&report, format)?;
            }
        } else {
            use std::io::Read;
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            let text = buf.trim().to_string();
            if !text.is_empty() {
                let report = pipeline.process(&text).map_err(|e| anyhow::anyhow!(e))?;
                print_report(&report, format)?;
            }
        }
        Ok(())
    }
}
