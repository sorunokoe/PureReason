//! # PureReason REST API Server (S-III-4 + S-IV-7)
//!
//! An axum-based HTTP server exposing the Kantian pipeline as a REST API.
//! Now includes: SLA monitoring (S-IV-7), compliance reporting (S-IV-1),
//! and structured decision validation (S-IV-9).
//!
//! ## Endpoints
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET    | /api/v1/health             | Health check |
//! | POST   | /api/v1/calibrate          | **Epistemic Confidence Score (ECS)** — primary endpoint |
//! | POST   | /api/v1/analyze            | Full pipeline analysis |
//! | POST   | /api/v1/claims             | Claim-first analysis with local evidence binding |
//! | POST   | /api/v1/certify            | Validation certificate |
//! | POST   | /api/v1/regulate           | Regulative transformation |
//! | POST   | /api/v1/validate           | Quick validation |
//! | POST   | /api/v1/compliance         | Regulatory compliance report |
//! | POST   | /api/v1/validate-decision  | Structured JSON decision validation |
//! | GET    | /api/v1/sla/report         | SLA compliance report |
//! | GET    | /api/v1/sla/status         | SLA status summary |
//! | POST   | /api/v1/trust/evaluate     | Persistent trust receipt + policy decision |
//! | GET    | /api/v1/trust/overview     | Trust ops overview metrics |
//! | GET    | /api/v1/trust/receipts     | Recent trust receipts |
//! | GET    | /api/v1/trust/reviews      | Review queue |
//! | GET    | /api/v1/trust/audit        | Audit history |
//! | GET    | /api/v1/trust/export       | Export trust ops bundle |

mod sla;

use axum::{
    extract::{DefaultBodyLimit, Json, Path, Query, Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Extension, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tracing::{error, info};

use pure_reason_core::{
    calibration::PipelineCalibration,
    certificate::ValidationCertificate,
    claims::annotate_claims,
    compliance::{ComplianceFramework, ComplianceReport},
    ensure_auth_configuration, is_disallowed_webhook_host,
    pipeline::KantianPipeline,
    rewriter::RewriteDomain,
    structured_validator::StructuredDecisionValidator,
    trust_ops::{
        AuditEvent, AuditEventKind, PolicyAction, ReviewUpdate, TrustOpsStore, TrustReceipt,
        TrustRole, DEFAULT_LIST_LIMIT, MAX_LIST_LIMIT,
    },
    ApiKeyRegistry, ApiPrincipal,
};

use sla::{SlaDefinition, SlaMonitor};

// ─── App State ───────────────────────────────────────────────────────────────

/// Maximum text input size (64 KB). Prevents O(n²) DoS via large inputs.
const MAX_TEXT_BYTES: usize = 65_536;
const MAX_REQUEST_BYTES: usize = MAX_TEXT_BYTES + 4_096;
const WEBHOOK_CONNECT_TIMEOUT_SECS: u64 = 5;
const WEBHOOK_TIMEOUT_SECS: u64 = 10;
const AUTH_RATE_WINDOW_SECS: u64 = 60;
const AUTH_MAX_FAILURES_PER_WINDOW: u32 = 20;

#[derive(Clone)]
struct AppState {
    pipeline: Arc<KantianPipeline>,
    sla_monitor: Arc<SlaMonitor>,
    api_keys: ApiKeyRegistry,
    ops_store: Arc<TrustOpsStore>,
    alert_webhooks: Arc<Vec<String>>,
    auth_rate_limiter: Arc<Mutex<AuthRateLimiter>>,
}

#[derive(Debug)]
struct AuthRateLimiter {
    window_started: Instant,
    failures: HashMap<IpAddr, u32>,
}

impl AuthRateLimiter {
    fn new() -> Self {
        Self {
            window_started: Instant::now(),
            failures: HashMap::new(),
        }
    }

    fn maybe_reset_window(&mut self) {
        if self.window_started.elapsed() >= Duration::from_secs(AUTH_RATE_WINDOW_SECS) {
            self.window_started = Instant::now();
            self.failures.clear();
        }
    }

    fn register_failure(&mut self, ip: IpAddr) -> bool {
        self.maybe_reset_window();
        let count = self.failures.entry(ip).or_insert(0);
        *count += 1;
        *count > AUTH_MAX_FAILURES_PER_WINDOW
    }

    fn clear(&mut self, ip: IpAddr) {
        self.maybe_reset_window();
        self.failures.remove(&ip);
    }
}

// ─── Pipeline helper ─────────────────────────────────────────────────────────

/// Run `KantianPipeline::process` on a blocking thread pool.
///
/// S28 (TRIZ Report XI): `process()` is CPU-heavy (linear scan over atlas,
/// regex passes, antinomy resolution). Calling it directly inside an async
/// handler starves the Tokio executor under concurrent load.
/// `spawn_blocking` hands it to a dedicated thread pool without blocking I/O.
async fn run_pipeline(
    pipeline: &Arc<KantianPipeline>,
    text: &str,
) -> Result<pure_reason_core::pipeline::PipelineReport, axum::response::Response> {
    let pipeline = Arc::clone(pipeline);
    let text = text.to_string();
    match tokio::task::spawn_blocking(move || pipeline.process(&text)).await {
        Ok(Ok(report)) => Ok(report),
        Ok(Err(e)) => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response()),
        Err(e) => {
            error!("pipeline task join error: {e}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal server error"})),
            )
                .into_response())
        }
    }
}

// ─── CLI ─────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "pure-reason-api", about = "PureReason REST API server")]
struct Cli {
    #[arg(long, default_value = "127.0.0.1:8080")]
    bind: String,

    #[arg(long)]
    ops_dir: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    allow_unauthenticated: bool,

    #[arg(long, env = "PURE_REASON_ALERT_WEBHOOKS", value_delimiter = ',')]
    alert_webhooks: Vec<String>,
}

// ─── Request / Response types ─────────────────────────────────────────────────

#[derive(Deserialize)]
struct TextRequest {
    text: String,
}

#[derive(Deserialize)]
struct CalibrateRequest {
    text: String,
    /// Optional domain for targeted regulative rewrite: medical, legal, financial, technical, general
    #[serde(default)]
    domain: String,
}

#[derive(Deserialize)]
struct ComplianceRequest {
    text: String,
    #[serde(default)]
    framework: String,
}

#[derive(Deserialize)]
struct DecisionRequest {
    json: String,
    #[serde(default)]
    domain: String,
}

#[derive(Deserialize)]
struct TrustEvaluateRequest {
    text: String,
    #[serde(default)]
    domain: String,
}

#[derive(Deserialize)]
struct ListQuery {
    #[serde(default = "default_limit")]
    limit: usize,
}

impl ListQuery {
    fn effective_limit(&self) -> usize {
        if self.limit == 0 {
            DEFAULT_LIST_LIMIT
        } else {
            self.limit.min(MAX_LIST_LIMIT)
        }
    }
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
    name: &'static str,
}

#[derive(Serialize)]
struct RegulateResponse {
    regulated_text: String,
    transformations_count: usize,
    risk_level: String,
}

#[derive(Serialize)]
struct ValidateResponse {
    has_illusions: bool,
    has_contradictions: bool,
    has_paralogisms: bool,
    risk_level: String,
    summary: String,
}

fn default_limit() -> usize {
    DEFAULT_LIST_LIMIT
}

// ─── Auth Middleware ──────────────────────────────────────────────────────────

/// Bearer token authentication middleware.
///
/// When local access tokens are configured, every request must carry a valid
/// `Authorization: Bearer <token>` header. When token auth is disabled, the API
/// falls back to local-admin mode.
///
/// Returns `401 Unauthorized` for missing or invalid tokens when auth is enabled.
async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> impl IntoResponse {
    if !state.api_keys.auth_enabled {
        request.extensions_mut().insert(ApiPrincipal::local_admin());
        return next.run(request).await.into_response();
    }

    let client_ip = request
        .extensions()
        .get::<axum::extract::ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip())
        .unwrap_or(IpAddr::from([0, 0, 0, 0]));

    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match token {
        Some(key) => match state.api_keys.validate(key) {
            Some(principal) => {
                if let Ok(mut limiter) = state.auth_rate_limiter.lock() {
                    limiter.clear(client_ip);
                }
                request.extensions_mut().insert(principal);
                next.run(request).await.into_response()
            }
            None => {
                if let Ok(mut limiter) = state.auth_rate_limiter.lock() {
                    if limiter.register_failure(client_ip) {
                        return (
                            StatusCode::TOO_MANY_REQUESTS,
                            Json(serde_json::json!({
                                "error": "Too many authentication failures. Retry later."
                            })),
                        )
                            .into_response();
                    }
                }
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": "Unauthorized. Provide a valid Bearer token in the Authorization header."
                    })),
                )
                    .into_response()
            }
        },
        _ => {
            if let Ok(mut limiter) = state.auth_rate_limiter.lock() {
                if limiter.register_failure(client_ip) {
                    return (
                        StatusCode::TOO_MANY_REQUESTS,
                        Json(serde_json::json!({
                            "error": "Too many authentication failures. Retry later."
                        })),
                    )
                        .into_response();
                }
            }
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Unauthorized. Provide a valid Bearer token in the Authorization header."
                })),
            ).into_response()
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn internal_server_error(
    context: &str,
    error_value: &impl std::fmt::Display,
) -> axum::response::Response {
    error!("{context}: {error_value}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "Internal server error" })),
    )
        .into_response()
}

/// Return 413 if `text` exceeds MAX_TEXT_BYTES.
fn check_size(text: &str) -> Option<axum::response::Response> {
    if text.len() > MAX_TEXT_BYTES {
        Some((
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(serde_json::json!({
                "error": format!("Input exceeds maximum allowed size of {} bytes", MAX_TEXT_BYTES)
            })),
        ).into_response())
    } else {
        None
    }
}

/// Serialize `value` to JSON, returning a 500 on failure.
// The Err variant (axum::response::Response) is intentionally large — it is an infrequent
// error path and boxing would add unnecessary indirection. Suppress the lint here.
#[allow(clippy::result_large_err)]
fn to_json<T: Serialize>(value: &T) -> Result<serde_json::Value, axum::response::Response> {
    serde_json::to_value(value).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Serialization error: {}", e)})),
        )
            .into_response()
    })
}

fn forbidden(message: &str) -> axum::response::Response {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": message })),
    )
        .into_response()
}

fn role_guard(principal: &ApiPrincipal) -> Option<axum::response::Response> {
    if principal.role.can_review() {
        None
    } else {
        Some(forbidden("This endpoint requires reviewer or admin role."))
    }
}

fn operator_guard(principal: &ApiPrincipal) -> Option<axum::response::Response> {
    if principal.role.can_operate() {
        None
    } else {
        Some(forbidden(
            "This endpoint requires operator, reviewer, or admin role.",
        ))
    }
}

#[derive(Clone)]
struct WebhookTarget {
    url: reqwest::Url,
    host: String,
    resolved_addrs: Vec<SocketAddr>,
}

async fn validate_webhook_url(url: &str) -> Result<WebhookTarget, String> {
    let parsed = reqwest::Url::parse(url).map_err(|error| error.to_string())?;
    // S32: Enforce HTTPS-only to prevent SSRF via plaintext interception.
    if parsed.scheme() != "https" {
        return Err("webhooks must use https (plaintext http is not permitted)".to_string());
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| "webhook URL must include a host".to_string())?
        .to_string();
    if is_disallowed_webhook_host(&host) {
        return Err("webhooks must not target localhost or private IP ranges".to_string());
    }

    let port = parsed
        .port_or_known_default()
        .ok_or_else(|| "webhook URL must use a supported port".to_string())?;
    // S32: Resolve host once, then pin the resolved IP for the connection.
    // This closes the DNS-rebinding window: we validate and record the IP
    // at registration time rather than trusting a re-resolved name later.
    let resolved = tokio::net::lookup_host((host.as_str(), port))
        .await
        .map_err(|error| format!("webhook host resolution failed: {error}"))?;
    let mut resolved_addrs = Vec::new();

    for addr in resolved {
        if is_disallowed_webhook_host(&addr.ip().to_string()) {
            return Err("webhooks must not resolve to localhost or private IP ranges".to_string());
        }
        resolved_addrs.push(addr);
    }

    if resolved_addrs.is_empty() {
        return Err("webhook host resolved to no addresses".to_string());
    }

    Ok(WebhookTarget {
        url: parsed,
        host,
        resolved_addrs,
    })
}

fn build_pinned_webhook_client(target: &WebhookTarget) -> Result<reqwest::Client, reqwest::Error> {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(WEBHOOK_CONNECT_TIMEOUT_SECS))
        .timeout(Duration::from_secs(WEBHOOK_TIMEOUT_SECS))
        .pool_max_idle_per_host(2);
    for addr in &target.resolved_addrs {
        builder = builder.resolve(target.host.as_str(), *addr);
    }
    builder.build()
}

#[derive(Serialize)]
struct WebhookPayload {
    tenant: String,
    role: TrustRole,
    receipt_id: String,
    risk_level: String,
    action: PolicyAction,
    review_required: bool,
}

async fn dispatch_webhooks(
    ops_store: Arc<TrustOpsStore>,
    webhook_urls: Arc<Vec<String>>,
    principal: ApiPrincipal,
    receipt: TrustReceipt,
) {
    if webhook_urls.is_empty() {
        return;
    }

    // Outbound webhook delivery is gated behind the `webhooks` cargo feature
    // (TRIZ-42 NE-12). Default builds make no outbound HTTP requests; URL
    // validation still runs so misconfigured webhooks show up in the audit log
    // with a clear reason.
    #[cfg(not(feature = "webhooks"))]
    {
        for url in webhook_urls.iter() {
            let _ = ops_store.append_audit_event(&AuditEvent::new(
                principal.tenant.clone(),
                principal.actor_id(),
                AuditEventKind::WebhookFailed,
                receipt.receipt_id.clone(),
                "Alert webhook disabled (build without `webhooks` feature)",
                serde_json::json!({ "url": url, "feature": "webhooks", "enabled": false }),
            ));
        }
        return;
    }

    #[allow(unreachable_code)]
    let payload = WebhookPayload {
        tenant: principal.tenant.clone(),
        role: principal.role,
        receipt_id: receipt.receipt_id.clone(),
        risk_level: receipt.risk_level.clone(),
        action: receipt.policy_decision.action,
        review_required: receipt.policy_decision.review_required,
    };

    for url in webhook_urls.iter() {
        let target = match validate_webhook_url(url).await {
            Ok(target) => target,
            Err(error) => {
                let _ = ops_store.append_audit_event(&AuditEvent::new(
                    principal.tenant.clone(),
                    principal.actor_id(),
                    AuditEventKind::WebhookFailed,
                    receipt.receipt_id.clone(),
                    "Alert webhook rejected",
                    serde_json::json!({ "url": url, "error": error }),
                ));
                continue;
            }
        };

        let client = match build_pinned_webhook_client(&target) {
            Ok(client) => client,
            Err(error) => {
                let _ = ops_store.append_audit_event(&AuditEvent::new(
                    principal.tenant.clone(),
                    principal.actor_id(),
                    AuditEventKind::WebhookFailed,
                    receipt.receipt_id.clone(),
                    "Alert webhook client initialization failed",
                    serde_json::json!({ "url": url, "error": error.to_string() }),
                ));
                continue;
            }
        };

        let result = client.post(target.url.clone()).json(&payload).send().await;

        let (kind, message, metadata) = match result {
            Ok(response) => (
                AuditEventKind::WebhookSent,
                "Alert webhook delivered".to_string(),
                serde_json::json!({ "url": url, "status": response.status().as_u16() }),
            ),
            Err(error) => (
                AuditEventKind::WebhookFailed,
                "Alert webhook failed".to_string(),
                serde_json::json!({ "url": url, "error": error.to_string() }),
            ),
        };

        let _ = ops_store.append_audit_event(&AuditEvent::new(
            principal.tenant.clone(),
            principal.actor_id(),
            kind,
            receipt.receipt_id.clone(),
            message,
            metadata,
        ));
    }
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        name: "pure-reason-api",
    })
}

async fn analyze(State(state): State<AppState>, Json(req): Json<TextRequest>) -> impl IntoResponse {
    if let Some(err) = check_size(&req.text) {
        return err;
    }
    match run_pipeline(&state.pipeline, &req.text).await {
        Ok(report) => {
            state.sla_monitor.record(&report);
            match to_json(&report) {
                Ok(v) => (StatusCode::OK, Json(v)).into_response(),
                Err(e) => e,
            }
        }
        Err(e) => e,
    }
}

async fn claims(Json(req): Json<TextRequest>) -> impl IntoResponse {
    if let Some(err) = check_size(&req.text) {
        return err;
    }
    match annotate_claims(&req.text) {
        Ok(report) => match to_json(&report) {
            Ok(v) => (StatusCode::OK, Json(v)).into_response(),
            Err(e) => e,
        },
        Err(e) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn certify(State(state): State<AppState>, Json(req): Json<TextRequest>) -> impl IntoResponse {
    if let Some(err) = check_size(&req.text) {
        return err;
    }
    match run_pipeline(&state.pipeline, &req.text).await {
        Ok(report) => {
            let cert = ValidationCertificate::from_report(&report);
            match to_json(&cert) {
                Ok(v) => (StatusCode::OK, Json(v)).into_response(),
                Err(e) => e,
            }
        }
        Err(e) => e,
    }
}

async fn regulate(
    State(state): State<AppState>,
    Json(req): Json<TextRequest>,
) -> impl IntoResponse {
    if let Some(err) = check_size(&req.text) {
        return err;
    }
    match run_pipeline(&state.pipeline, &req.text).await {
        Ok(report) => {
            let resp = RegulateResponse {
                regulated_text: report.regulated_text.clone(),
                transformations_count: report.transformations.len(),
                risk_level: report.verdict.risk.to_string(),
            };
            match to_json(&resp) {
                Ok(v) => (StatusCode::OK, Json(v)).into_response(),
                Err(e) => e,
            }
        }
        Err(e) => e,
    }
}

async fn validate(
    State(state): State<AppState>,
    Json(req): Json<TextRequest>,
) -> impl IntoResponse {
    if let Some(err) = check_size(&req.text) {
        return err;
    }
    match run_pipeline(&state.pipeline, &req.text).await {
        Ok(report) => {
            state.sla_monitor.record(&report);
            let resp = ValidateResponse {
                has_illusions: report.verdict.has_illusions,
                has_contradictions: report.verdict.has_contradictions,
                has_paralogisms: report.verdict.has_paralogisms,
                risk_level: report.verdict.risk.to_string(),
                summary: report.summary.clone(),
            };
            match to_json(&resp) {
                Ok(v) => (StatusCode::OK, Json(v)).into_response(),
                Err(e) => e,
            }
        }
        Err(e) => e,
    }
}

async fn compliance(
    State(state): State<AppState>,
    Json(req): Json<ComplianceRequest>,
) -> impl IntoResponse {
    if let Some(err) = check_size(&req.text) {
        return err;
    }
    let framework = match req.framework.to_lowercase().as_str() {
        "hipaa" => ComplianceFramework::Hipaa,
        "sec" | "sec_rule_10b5" | "sec-rule-10b5" => ComplianceFramework::SecRule10b5,
        "fda" | "fda_ai_ml" => ComplianceFramework::FdaAiMlGuidance,
        "nist" | "nist_ai_rmf" => ComplianceFramework::NistAiRmf,
        "gdpr" => ComplianceFramework::Gdpr,
        _ => ComplianceFramework::EuAiAct, // default
    };
    match run_pipeline(&state.pipeline, &req.text).await {
        Ok(report) => {
            let compliance_report = ComplianceReport::generate(&report, framework);
            match to_json(&compliance_report) {
                Ok(mut v) => {
                    // Add heuristic disclaimer — compliance output is keyword-based,
                    // not a legal opinion. Do not cite in regulatory submissions.
                    v["disclaimer"] = serde_json::json!(
                        "HEURISTIC GUIDANCE ONLY. This report is generated by automated \
                         pattern analysis and does not constitute legal advice or a formal \
                         regulatory assessment. Consult qualified legal counsel before \
                         relying on this output for compliance purposes."
                    );
                    (StatusCode::OK, Json(v)).into_response()
                }
                Err(e) => e,
            }
        }
        Err(e) => e,
    }
}

async fn validate_decision(Json(req): Json<DecisionRequest>) -> impl IntoResponse {
    if let Some(err) = check_size(&req.json) {
        return err;
    }
    let json = req.json.clone();
    let domain = req.domain.clone();
    match tokio::task::spawn_blocking(move || {
        let validator = if domain.is_empty() {
            StructuredDecisionValidator::new()
        } else {
            StructuredDecisionValidator::with_domain(&domain)
        };
        validator.validate_json(&json)
    })
    .await
    {
        Ok(Ok(result)) => match to_json(&result) {
            Ok(v) => (StatusCode::OK, Json(v)).into_response(),
            Err(e) => e,
        },
        Ok(Err(e)) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
        Err(e) => internal_server_error("structured decision validation task panicked", &e),
    }
}

async fn calibrate(
    State(state): State<AppState>,
    Json(req): Json<CalibrateRequest>,
) -> impl IntoResponse {
    if let Some(err) = check_size(&req.text) {
        return err;
    }
    match run_pipeline(&state.pipeline, &req.text).await {
        Ok(report) => {
            let cal = report.calibration();
            let domain = RewriteDomain::parse_domain(&req.domain);
            let domain_rewrite = if !cal.calibrated {
                let r = cal.rewrite_for_domain(domain);
                if r.changed {
                    Some(r)
                } else {
                    None
                }
            } else {
                None
            };
            let mut v = match to_json(&cal) {
                Ok(v) => v,
                Err(e) => return e,
            };
            if let Some(dr) = domain_rewrite {
                if let Ok(dr_val) = serde_json::to_value(&dr) {
                    v["domain_rewrite"] = dr_val;
                }
            }
            (StatusCode::OK, Json(v)).into_response()
        }
        Err(e) => e,
    }
}

async fn sla_report(State(state): State<AppState>) -> impl IntoResponse {
    let report = state.sla_monitor.report();
    match to_json(&report) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => e,
    }
}

async fn sla_status(State(state): State<AppState>) -> impl IntoResponse {
    let report = state.sla_monitor.report();
    // Note: auto_regulated_count is a factual metric; economic impact is out of scope.
    Json(serde_json::json!({
        "compliant": report.overall_compliant,
        "health_score": report.epistemic_health_score,
        "total_requests": report.total_requests,
        "auto_regulated_count": report.auto_regulated_count,
    }))
    .into_response()
}

async fn trust_evaluate(
    State(state): State<AppState>,
    Extension(principal): Extension<ApiPrincipal>,
    Json(req): Json<TrustEvaluateRequest>,
) -> impl IntoResponse {
    if let Some(response) = operator_guard(&principal) {
        return response;
    }
    if let Some(err) = check_size(&req.text) {
        return err;
    }

    let domain = if req.domain.is_empty() {
        None
    } else {
        Some(req.domain.as_str())
    };

    match run_pipeline(&state.pipeline, &req.text).await {
        Ok(report) => {
            state.sla_monitor.record(&report);
            let evaluation = match state.ops_store.record_report(
                &report,
                Some(&principal.tenant),
                Some(&principal.actor_id()),
                domain,
            ) {
                Ok(value) => value,
                Err(error) => {
                    return internal_server_error("failed to persist trust evaluation", &error)
                }
            };

            if evaluation.receipt.policy_decision.review_required {
                let ops_store = state.ops_store.clone();
                let webhook_urls = state.alert_webhooks.clone();
                let principal = principal.clone();
                let receipt = evaluation.receipt.clone();
                tokio::spawn(async move {
                    dispatch_webhooks(ops_store, webhook_urls, principal, receipt).await;
                });
            }

            Json(serde_json::json!({
                "report": report,
                "receipt": evaluation.receipt,
                "review_item": evaluation.review_item,
            }))
            .into_response()
        }
        Err(e) => e,
    }
}

async fn trust_overview(
    State(state): State<AppState>,
    Extension(principal): Extension<ApiPrincipal>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    match state
        .ops_store
        .overview(Some(&principal.tenant), query.effective_limit())
    {
        Ok(overview) => match to_json(&overview) {
            Ok(value) => (StatusCode::OK, Json(value)).into_response(),
            Err(error) => error,
        },
        Err(error) => internal_server_error("failed to build trust overview", &error),
    }
}

async fn trust_receipts(
    State(state): State<AppState>,
    Extension(principal): Extension<ApiPrincipal>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    match state
        .ops_store
        .list_receipts(Some(&principal.tenant), query.effective_limit())
    {
        Ok(receipts) => match to_json(&receipts) {
            Ok(value) => (StatusCode::OK, Json(value)).into_response(),
            Err(error) => error,
        },
        Err(error) => internal_server_error("failed to list trust receipts", &error),
    }
}

async fn trust_receipt(
    Path(receipt_id): Path<String>,
    State(state): State<AppState>,
    Extension(principal): Extension<ApiPrincipal>,
) -> impl IntoResponse {
    match state
        .ops_store
        .get_receipt(&receipt_id, Some(&principal.tenant))
    {
        Ok(Some(receipt)) => match to_json(&receipt) {
            Ok(value) => (StatusCode::OK, Json(value)).into_response(),
            Err(error) => error,
        },
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Receipt not found" })),
        )
            .into_response(),
        Err(error) => internal_server_error("failed to fetch trust receipt", &error),
    }
}

async fn trust_reviews(
    State(state): State<AppState>,
    Extension(principal): Extension<ApiPrincipal>,
) -> impl IntoResponse {
    match state.ops_store.list_reviews(Some(&principal.tenant)) {
        Ok(reviews) => match to_json(&reviews) {
            Ok(value) => (StatusCode::OK, Json(value)).into_response(),
            Err(error) => error,
        },
        Err(error) => internal_server_error("failed to list trust reviews", &error),
    }
}

async fn trust_review_update(
    Path(review_id): Path<String>,
    State(state): State<AppState>,
    Extension(principal): Extension<ApiPrincipal>,
    Json(update): Json<ReviewUpdate>,
) -> impl IntoResponse {
    if let Some(response) = role_guard(&principal) {
        return response;
    }

    match state.ops_store.update_review(
        &review_id,
        Some(&principal.tenant),
        &principal.actor_id(),
        update,
    ) {
        Ok(Some(review)) => match to_json(&review) {
            Ok(value) => (StatusCode::OK, Json(value)).into_response(),
            Err(error) => error,
        },
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Review item not found" })),
        )
            .into_response(),
        Err(error) => internal_server_error("failed to update trust review", &error),
    }
}

async fn trust_audit(
    State(state): State<AppState>,
    Extension(principal): Extension<ApiPrincipal>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    match state
        .ops_store
        .list_audit_events(Some(&principal.tenant), query.effective_limit())
    {
        Ok(events) => match to_json(&events) {
            Ok(value) => (StatusCode::OK, Json(value)).into_response(),
            Err(error) => error,
        },
        Err(error) => internal_server_error("failed to list audit events", &error),
    }
}

async fn trust_export(
    State(state): State<AppState>,
    Extension(principal): Extension<ApiPrincipal>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    match state
        .ops_store
        .export_bundle(Some(&principal.tenant), query.effective_limit())
    {
        Ok(bundle) => match to_json(&bundle) {
            Ok(value) => (StatusCode::OK, Json(value)).into_response(),
            Err(error) => error,
        },
        Err(error) => internal_server_error("failed to export trust bundle", &error),
    }
}

// ─── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let api_keys = ApiKeyRegistry::from_env();
    if let Err(error) = ensure_auth_configuration(
        "pure-reason-api",
        &cli.bind,
        api_keys.auth_enabled,
        cli.allow_unauthenticated,
    ) {
        eprintln!("{error}");
        std::process::exit(2);
    }

    if !api_keys.auth_enabled {
        info!("Running pure-reason-api without API-key auth; intended for local or explicitly trusted environments");
    }

    let ops_store = match cli.ops_dir.clone() {
        Some(path) => TrustOpsStore::with_base(path),
        None => TrustOpsStore::new(),
    };
    let ops_store = match ops_store {
        Ok(store) => store,
        Err(error) => {
            eprintln!("failed to initialize trust ops store: {error}");
            std::process::exit(2);
        }
    };
    let state = AppState {
        pipeline: Arc::new(KantianPipeline::new()),
        sla_monitor: Arc::new(SlaMonitor::new(SlaDefinition::default())),
        api_keys,
        ops_store: Arc::new(ops_store),
        alert_webhooks: Arc::new(cli.alert_webhooks.clone()),
        auth_rate_limiter: Arc::new(Mutex::new(AuthRateLimiter::new())),
    };

    let protected_routes = Router::new()
        .route("/api/v1/analyze", post(analyze))
        .route("/api/v1/claims", post(claims))
        .route("/api/v1/certify", post(certify))
        .route("/api/v1/regulate", post(regulate))
        .route("/api/v1/validate", post(validate))
        .route("/api/v1/compliance", post(compliance))
        .route("/api/v1/validate-decision", post(validate_decision))
        .route("/api/v1/calibrate", post(calibrate))
        .route("/api/v1/sla/report", get(sla_report))
        .route("/api/v1/sla/status", get(sla_status))
        .route("/api/v1/trust/evaluate", post(trust_evaluate))
        .route("/api/v1/trust/overview", get(trust_overview))
        .route("/api/v1/trust/receipts", get(trust_receipts))
        .route("/api/v1/trust/receipts/:receipt_id", get(trust_receipt))
        .route("/api/v1/trust/reviews", get(trust_reviews))
        .route(
            "/api/v1/trust/reviews/:review_id",
            post(trust_review_update),
        )
        .route("/api/v1/trust/audit", get(trust_audit))
        .route("/api/v1/trust/export", get(trust_export))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let app = Router::new()
        .route("/api/v1/health", get(health))
        .merge(protected_routes)
        .with_state(state);

    let listener = match tokio::net::TcpListener::bind(&cli.bind).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("Failed to bind to {}: {}", cli.bind, error);
            std::process::exit(2);
        }
    };

    info!("PureReason API server listening on http://{}", cli.bind);
    if let Err(error) = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    {
        eprintln!("Server error: {}", error);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn validate_webhook_url_rejects_loopback_targets() {
        assert!(validate_webhook_url("https://127.0.0.1:8080")
            .await
            .is_err());
        assert!(validate_webhook_url("https://localhost:8080")
            .await
            .is_err());
    }

    #[tokio::test]
    async fn validate_webhook_url_rejects_non_https_schemes() {
        assert!(validate_webhook_url("ftp://example.com/alerts")
            .await
            .is_err());
        // S32: plain http must also be rejected
        assert!(validate_webhook_url("http://example.com/alerts")
            .await
            .is_err());
    }
}
