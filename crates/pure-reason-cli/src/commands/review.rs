use super::get_text;
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use pure_reason_memory::{EvidenceRecord, EvidenceStore};
use pure_reason_runtime::{
    executor::{StepOutcome, WorkflowExecutor},
    local_state::{open_local_stores, resolve_local_state_dir},
    WorkflowKind,
};
use pure_reason_trace::TraceId;
use pure_reason_verifier::{
    ArtifactKind, VerificationRequest, VerificationResult, VerifierService,
};
use serde::Serialize;
use std::path::PathBuf;

/// Review an artifact through the local verifier/runtime flow.
#[derive(Args)]
pub struct ReviewCmd {
    /// Artifact text to review (reads from stdin if omitted)
    pub text: Option<String>,

    /// Treat the input as a structured JSON decision instead of plain text
    #[arg(long, default_value_t = false)]
    pub structured: bool,

    /// Optional task description stored in the runtime task record
    #[arg(long)]
    pub description: Option<String>,

    /// Directory for persistent local task and trace state
    #[arg(long, env = "PURE_REASON_STATE_DIR")]
    pub state_dir: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct ReviewOutput {
    task_id: String,
    trace_id: String,
    final_state: String,
    verification: Option<VerificationResult>,
    error: Option<String>,
}

impl ReviewCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        let content = get_text(&self.text)?;
        let (verifier, executor, evidence_store) = open_review_runtime(self.state_dir.clone())?;

        let artifact_kind = if self.structured {
            ArtifactKind::StructuredDecision
        } else {
            ArtifactKind::Text
        };
        let description = self.description.clone().unwrap_or_else(|| {
            if self.structured {
                "Structured decision review".to_string()
            } else {
                "Text review".to_string()
            }
        });

        let trace_id = TraceId::new();
        let task = executor.create_task(trace_id, WorkflowKind::Verification, description)?;
        executor.begin(&task.task_id)?;
        let verification = verifier.verify(VerificationRequest {
            content: content.clone(),
            kind: artifact_kind.clone(),
            trace_id: Some(trace_id.to_string()),
        });

        let output = match verification {
            Ok(result) => {
                let outcome = if result.verdict.passed {
                    StepOutcome::Completed
                } else {
                    StepOutcome::NeedsReview
                };
                let final_state = outcome.terminal_state();
                let output = ReviewOutput {
                    task_id: task.task_id.to_string(),
                    trace_id: trace_id.to_string(),
                    final_state: final_state.to_string(),
                    verification: Some(result),
                    error: None,
                };

                let evidence = EvidenceRecord::from_review(
                    task.task_id,
                    trace_id,
                    artifact_kind.clone(),
                    output.final_state.clone(),
                    content.clone(),
                    output.verification.as_ref(),
                    output.error.clone(),
                );
                evidence_store.append(&evidence)?;
                executor.finish(&task.task_id, outcome)?;
                output
            }
            Err(error) => {
                let message = error.to_string();
                let outcome = StepOutcome::Error(message.clone());
                let final_state = outcome.terminal_state();
                let output = ReviewOutput {
                    task_id: task.task_id.to_string(),
                    trace_id: trace_id.to_string(),
                    final_state: final_state.to_string(),
                    verification: None,
                    error: Some(message),
                };

                let evidence = EvidenceRecord::from_review(
                    task.task_id,
                    trace_id,
                    artifact_kind.clone(),
                    output.final_state.clone(),
                    content.clone(),
                    output.verification.as_ref(),
                    output.error.clone(),
                );
                evidence_store.append(&evidence)?;
                executor.finish(&task.task_id, outcome)?;
                output
            }
        };

        match format {
            "json" => println!("{}", serde_json::to_string_pretty(&output)?),
            "markdown" => print_markdown(&output),
            _ => print_plain(&output),
        }

        Ok(())
    }
}

fn open_review_runtime(
    state_dir: Option<PathBuf>,
) -> Result<(VerifierService, WorkflowExecutor, EvidenceStore)> {
    let state_dir = resolve_local_state_dir(state_dir)?;
    let (task_store, trace_store) = open_local_stores(&state_dir)?;
    let evidence_store = EvidenceStore::open_local(&state_dir)?;
    let verifier = VerifierService::new().with_trace_store(trace_store.clone());
    let executor = WorkflowExecutor::new(task_store, trace_store);
    Ok((verifier, executor, evidence_store))
}

fn print_plain(output: &ReviewOutput) {
    println!("{}", "━".repeat(60).dimmed());
    println!("{}", "  PURE-REASON LOCAL REVIEW".bold().cyan());
    println!("{}", "━".repeat(60).dimmed());
    println!("{} {}", "Task ID:".bold(), output.task_id);
    println!("{} {}", "Trace ID:".bold(), output.trace_id);
    println!("{} {}", "Final State:".bold(), output.final_state);

    if let Some(error) = &output.error {
        println!("{} {}", "Error:".bold().red(), error);
    }

    if let Some(verification) = &output.verification {
        let verdict = if verification.verdict.passed {
            "PASSED".green().bold()
        } else {
            "REVIEW".yellow().bold()
        };
        println!("{} {}", "Verdict:".bold(), verdict);
        println!(
            "{} {:.2}",
            "Risk Score:".bold(),
            verification.verdict.risk_score
        );
        println!("{} {}", "Summary:".bold(), verification.verdict.summary);

        if let Some(regulated) = &verification.regulated_text {
            println!();
            println!("{}", "Regulated Text:".bold().underline());
            println!("{regulated}");
        }

        if !verification.findings.is_empty() {
            println!();
            println!("{}", "Findings:".bold().underline());
            for finding in &verification.findings {
                println!(
                    "  - [{:?}/{:?}] {}",
                    finding.severity, finding.category, finding.message
                );
            }
        }
    }

    println!("{}", "━".repeat(60).dimmed());
}

fn print_markdown(output: &ReviewOutput) {
    println!("# PureReason Local Review\n");
    println!("**Task ID:** `{}`  ", output.task_id);
    println!("**Trace ID:** `{}`  ", output.trace_id);
    println!("**Final State:** `{}`\n", output.final_state);

    if let Some(error) = &output.error {
        println!("**Error:** {}\n", error);
    }

    if let Some(verification) = &output.verification {
        println!(
            "**Passed:** {}  \n**Risk Score:** {:.2}  \n**Summary:** {}\n",
            verification.verdict.passed,
            verification.verdict.risk_score,
            verification.verdict.summary
        );

        if !verification.findings.is_empty() {
            println!("## Findings\n");
            for finding in &verification.findings {
                println!(
                    "- **{:?} / {:?}** — {}",
                    finding.severity, finding.category, finding.message
                );
            }
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pure_reason_memory::EvidenceStore;
    use pure_reason_runtime::open_local_stores;
    use tempfile::TempDir;

    #[test]
    fn review_runtime_persists_to_requested_state_dir() {
        let temp_dir = TempDir::new().unwrap();
        let state_dir = temp_dir.path().join("review-state");
        let (verifier, executor, evidence_store) =
            open_review_runtime(Some(state_dir.clone())).unwrap();

        let trace_id = TraceId::new();
        let task = executor
            .create_task(
                trace_id,
                WorkflowKind::Verification,
                "Persistent CLI review",
            )
            .unwrap();
        let verification = verifier
            .verify(VerificationRequest {
                content: "The sky is blue.".to_string(),
                kind: ArtifactKind::Text,
                trace_id: Some(trace_id.to_string()),
            })
            .unwrap();

        executor.begin(&task.task_id).unwrap();
        let outcome = if verification.verdict.passed {
            StepOutcome::Completed
        } else {
            StepOutcome::NeedsReview
        };
        let final_state = outcome.terminal_state();

        let evidence = EvidenceRecord::from_review(
            task.task_id,
            trace_id,
            ArtifactKind::Text,
            final_state.to_string(),
            "The sky is blue.",
            Some(&verification),
            None,
        );
        evidence_store.append(&evidence).unwrap();
        executor.finish(&task.task_id, outcome).unwrap();

        let (task_store, trace_store) = open_local_stores(&state_dir).unwrap();
        let evidence_store = EvidenceStore::open_local(&state_dir).unwrap();
        assert_eq!(task_store.list_recent(10).unwrap().len(), 1);
        assert!(!trace_store.list_by_trace(&trace_id, 10).unwrap().is_empty());
        assert_eq!(evidence_store.list_recent(10).unwrap().len(), 1);
    }
}
