//! # PureReason MCP Server (S-III-2)
//!
//! A Model Context Protocol (MCP) server that exposes PureReason as a local
//! assurance layer for frontier agents over JSON-RPC 2.0 via stdio.
//!
//! ## Tools
//! | Tool | Description |
//! |------|-------------|
//! | `analyze` | Full Kantian pipeline analysis |
//! | `certify` | Generate a content-addressed validation certificate |
//! | `regulate` | Convert constitutive overreach to regulative form |
//! | `validate` | Quick dialectical validation |
//! | `verify_text` | Run the local verifier service on plain text |
//! | `verify_structured_decision` | Run the local verifier service on JSON decisions |
//! | `review_text` | Create a verification task, run the verifier, and return the task state |
//! | `review_structured_decision` | Same review flow for structured JSON decisions |
//!
//! ## Usage (Claude Desktop config)
//! ```json
//! {
//!   "mcpServers": {
//!     "pure-reason": {
//!       "command": "pure-reason-mcp"
//!     }
//!   }
//! }
//! ```

use anyhow::Result;
use pure_reason_memory::{EvidenceRecord, EvidenceStore};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::PathBuf;

use pure_reason_core::{certificate::ValidationCertificate, pipeline::KantianPipeline};
use pure_reason_runtime::{
    executor::{StepOutcome, WorkflowExecutor},
    local_state::{open_local_stores, resolve_local_state_dir},
    WorkflowKind,
};
use pure_reason_trace::ids::TraceId;
use pure_reason_verifier::{
    ArtifactKind, VerificationRequest, VerificationResult, VerifierService,
};

// ─── Server state ───────────────────────────────────────────────────────────────

struct ServerState {
    pipeline: KantianPipeline,
    verifier: VerifierService,
    executor: WorkflowExecutor,
    evidence_store: EvidenceStore,
}

#[derive(Debug, Serialize)]
struct ReviewToolResult {
    task_id: String,
    trace_id: String,
    final_state: String,
    verification: Option<VerificationResult>,
    error: Option<String>,
}

// ─── JSON-RPC 2.0 types ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct Request {
    #[serde(rename = "jsonrpc")]
    _jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct Response {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i64,
    message: String,
}

impl Response {
    fn ok(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    fn err(id: Value, code: i64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
            }),
        }
    }
}

// ─── Tool definitions ─────────────────────────────────────────────────────────

fn tool_list() -> Value {
    json!({
        "tools": [
            {
                "name": "analyze",
                "description": "Run the full Kantian pipeline on text.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "The text to analyze" }
                    },
                    "required": ["text"]
                }
            },
            {
                "name": "certify",
                "description": "Generate a content-addressed ValidationCertificate for text.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "The text to certify" }
                    },
                    "required": ["text"]
                }
            },
            {
                "name": "regulate",
                "description": "Convert constitutive epistemic overreach in text to its regulative form.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "The text to regulate" }
                    },
                    "required": ["text"]
                }
            },
            {
                "name": "validate",
                "description": "Quick dialectical validation for plain text.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "The text to validate" }
                    },
                    "required": ["text"]
                }
            },
            {
                "name": "verify_text",
                "description": "Run the local verifier service on plain text and return a structured verification result.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "The text artifact to verify" }
                    },
                    "required": ["text"]
                }
            },
            {
                "name": "verify_structured_decision",
                "description": "Run the local verifier service on a structured JSON decision.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "json": { "type": "string", "description": "The JSON decision document to verify" }
                    },
                    "required": ["json"]
                }
            },
            {
                "name": "review_text",
                "description": "Create a verification task, run the local verifier on plain text, and return task + verdict state.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "The text artifact to review" }
                    },
                    "required": ["text"]
                }
            },
            {
                "name": "review_structured_decision",
                "description": "Create a verification task, run the local verifier on a structured JSON decision, and return task + verdict state.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "json": { "type": "string", "description": "The JSON decision document to review" }
                    },
                    "required": ["json"]
                }
            }
        ]
    })
}

// ─── Helpers ───────────────────────────────────────────────────────────────────

fn required_arg(args: &Value, key: &str) -> std::result::Result<String, String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("Missing required parameter: {key}"))
}

fn wrap_tool_result(id: Value, result_value: Value) -> Response {
    Response::ok(
        id,
        json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&result_value).unwrap_or_default()
            }]
        }),
    )
}

fn verify_artifact(
    state: &ServerState,
    kind: ArtifactKind,
    content: String,
) -> std::result::Result<Value, String> {
    let result = state
        .verifier
        .verify(VerificationRequest {
            content,
            kind,
            trace_id: None,
        })
        .map_err(|error| error.to_string())?;

    serde_json::to_value(result).map_err(|error| error.to_string())
}

fn review_artifact(
    state: &ServerState,
    kind: ArtifactKind,
    content: String,
) -> std::result::Result<Value, String> {
    let artifact_kind = kind.clone();
    let trace_id = TraceId::new();
    let description = match kind {
        ArtifactKind::Text => "MCP text review",
        ArtifactKind::StructuredDecision => "MCP structured decision review",
    };

    let task = state
        .executor
        .create_task(trace_id, WorkflowKind::Verification, description)
        .map_err(|error| error.to_string())?;
    state
        .executor
        .begin(&task.task_id)
        .map_err(|error| error.to_string())?;

    let verification = state.verifier.verify(VerificationRequest {
        content: content.clone(),
        kind,
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
            let output = ReviewToolResult {
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
            state
                .evidence_store
                .append(&evidence)
                .map_err(|error| error.to_string())?;
            state
                .executor
                .finish(&task.task_id, outcome)
                .map_err(|error| error.to_string())?;
            output
        }
        Err(error) => {
            let message = error.to_string();
            let outcome = StepOutcome::Error(message.clone());
            let final_state = outcome.terminal_state();
            let output = ReviewToolResult {
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
            state
                .evidence_store
                .append(&evidence)
                .map_err(|error| error.to_string())?;
            state
                .executor
                .finish(&task.task_id, outcome)
                .map_err(|run_error| run_error.to_string())?;
            output
        }
    };

    serde_json::to_value(output).map_err(|error| error.to_string())
}

// ─── Dispatch ─────────────────────────────────────────────────────────────────

fn handle_request(state: &ServerState, req: Request) -> Response {
    let id = req.id.unwrap_or(Value::Null);

    match req.method.as_str() {
        "initialize" => Response::ok(
            id,
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "pure-reason-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        ),
        "initialized" => Response::ok(id, json!({})),
        "tools/list" => Response::ok(id, tool_list()),
        "tools/call" => {
            let params = req.params.unwrap_or(json!({}));
            let tool_name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));

            let result_value = match tool_name {
                "analyze" | "certify" | "regulate" | "validate" => {
                    let text = match required_arg(&args, "text") {
                        Ok(text) => text,
                        Err(message) => return Response::err(id, -32602, message),
                    };

                    let report = match state.pipeline.process(&text) {
                        Ok(report) => report,
                        Err(error) => {
                            return Response::err(id, -32000, format!("Pipeline error: {error}"))
                        }
                    };

                    match tool_name {
                        "analyze" => serde_json::to_value(&report)
                            .unwrap_or(json!({"error": "serialization failed"})),
                        "certify" => {
                            let cert = ValidationCertificate::from_report(&report);
                            serde_json::to_value(&cert)
                                .unwrap_or(json!({"error": "serialization failed"}))
                        }
                        "regulate" => json!({
                            "regulated_text": report.regulated_text,
                            "transformations_count": report.transformations.len(),
                            "risk_level": report.verdict.risk.to_string()
                        }),
                        "validate" => json!({
                            "risk_level": report.verdict.risk.to_string(),
                            "has_illusions": report.verdict.has_illusions,
                            "has_contradictions": report.verdict.has_contradictions,
                            "has_paralogisms": report.verdict.has_paralogisms,
                            "summary": report.summary
                        }),
                        _ => unreachable!(),
                    }
                }
                "verify_text" => {
                    let text = match required_arg(&args, "text") {
                        Ok(text) => text,
                        Err(message) => return Response::err(id, -32602, message),
                    };
                    match verify_artifact(state, ArtifactKind::Text, text) {
                        Ok(value) => value,
                        Err(message) => return Response::err(id, -32000, message),
                    }
                }
                "verify_structured_decision" => {
                    let json_input = match required_arg(&args, "json") {
                        Ok(json_input) => json_input,
                        Err(message) => return Response::err(id, -32602, message),
                    };
                    match verify_artifact(state, ArtifactKind::StructuredDecision, json_input) {
                        Ok(value) => value,
                        Err(message) => return Response::err(id, -32000, message),
                    }
                }
                "review_text" => {
                    let text = match required_arg(&args, "text") {
                        Ok(text) => text,
                        Err(message) => return Response::err(id, -32602, message),
                    };
                    match review_artifact(state, ArtifactKind::Text, text) {
                        Ok(value) => value,
                        Err(message) => return Response::err(id, -32000, message),
                    }
                }
                "review_structured_decision" => {
                    let json_input = match required_arg(&args, "json") {
                        Ok(json_input) => json_input,
                        Err(message) => return Response::err(id, -32602, message),
                    };
                    match review_artifact(state, ArtifactKind::StructuredDecision, json_input) {
                        Ok(value) => value,
                        Err(message) => return Response::err(id, -32000, message),
                    }
                }
                unknown => return Response::err(id, -32601, format!("Unknown tool: {unknown}")),
            };

            wrap_tool_result(id, result_value)
        }
        "ping" => Response::ok(id, json!({})),
        unknown => Response::err(id, -32601, format!("Method not found: {unknown}")),
    }
}

// ─── Main: stdio JSON-RPC loop ────────────────────────────────────────────────

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    let state = build_server_state(None)?;

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) if l.trim().is_empty() => continue,
            Ok(l) => l,
            Err(_) => break,
        };

        let response = match serde_json::from_str::<Request>(&line) {
            Ok(req) => handle_request(&state, req),
            Err(error) => Response {
                jsonrpc: "2.0",
                id: Value::Null,
                result: None,
                error: Some(RpcError {
                    code: -32700,
                    message: format!("Parse error: {error}"),
                }),
            },
        };

        if response.id == Value::Null && response.error.is_none() {
            continue;
        }

        let mut json_line = serde_json::to_string(&response)?;
        json_line.push('\n');
        out.write_all(json_line.as_bytes())?;
        out.flush()?;
    }

    Ok(())
}

fn build_server_state(state_dir: Option<PathBuf>) -> Result<ServerState> {
    let state_dir = resolve_local_state_dir(state_dir)?;
    let (task_store, trace_store) = open_local_stores(&state_dir)?;
    let evidence_store = EvidenceStore::open_local(&state_dir)?;
    tracing::info!(
        state_dir = %state_dir.display(),
        "Using persistent local task and trace state"
    );
    let verifier = VerifierService::new().with_trace_store(trace_store.clone());
    let executor = WorkflowExecutor::new(task_store, trace_store);
    Ok(ServerState {
        pipeline: KantianPipeline::new(),
        verifier,
        executor,
        evidence_store,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pure_reason_memory::EvidenceStore;
    use pure_reason_runtime::open_local_stores;
    use tempfile::TempDir;

    fn test_state() -> ServerState {
        build_server_state(Some(
            std::env::temp_dir().join(format!("pure-reason-mcp-test-{}", uuid::Uuid::new_v4())),
        ))
        .unwrap()
    }

    fn tool_call(name: &str, arguments: Value) -> Request {
        Request {
            _jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": name,
                "arguments": arguments
            })),
        }
    }

    #[test]
    fn tools_list_includes_new_review_tools() {
        let state = test_state();
        let response = handle_request(
            &state,
            Request {
                _jsonrpc: "2.0".to_string(),
                id: Some(json!(1)),
                method: "tools/list".to_string(),
                params: None,
            },
        );

        let tools = response.result.unwrap()["tools"]
            .as_array()
            .unwrap()
            .clone();
        assert!(tools.iter().any(|tool| tool["name"] == "verify_text"));
        assert!(tools.iter().any(|tool| tool["name"] == "review_text"));
    }

    #[test]
    fn verify_text_tool_returns_structured_verdict() {
        let state = test_state();
        let response = handle_request(
            &state,
            tool_call(
                "verify_text",
                json!({"text": "Water boils at 100 degrees Celsius at sea level."}),
            ),
        );

        let body = response.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap()
            .to_string();
        let payload: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(payload["verdict"]["passed"], Value::Bool(true));
    }

    #[test]
    fn review_structured_decision_routes_to_review() {
        let state = test_state();
        let response = handle_request(
            &state,
            tool_call(
                "review_structured_decision",
                json!({
                    "json": "{\"contraindications\":[\"warfarin\"],\"prescribed\":[\"warfarin\"]}"
                }),
            ),
        );

        let body = response.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap()
            .to_string();
        let payload: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            payload["final_state"],
            Value::String("awaiting_review".to_string())
        );
        assert!(payload["verification"]["findings"]
            .as_array()
            .is_some_and(|findings| !findings.is_empty()));
    }

    #[test]
    fn server_state_persists_review_state_to_disk() {
        let temp_dir = TempDir::new().unwrap();
        let state_dir = temp_dir.path().join("mcp-state");
        let state = build_server_state(Some(state_dir.clone())).unwrap();

        let result =
            review_artifact(&state, ArtifactKind::Text, "The sky is blue.".to_string()).unwrap();
        let trace_id = result["trace_id"]
            .as_str()
            .unwrap()
            .parse::<TraceId>()
            .unwrap();

        let (task_store, trace_store) = open_local_stores(&state_dir).unwrap();
        let evidence_store = EvidenceStore::open_local(&state_dir).unwrap();
        assert_eq!(task_store.list_recent(10).unwrap().len(), 1);
        assert!(!trace_store.list_by_trace(&trace_id, 10).unwrap().is_empty());
        assert_eq!(evidence_store.list_recent(10).unwrap().len(), 1);
    }
}
