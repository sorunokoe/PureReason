//! # PureReason Trust Operations Console
//!
//! Persistent dashboard for trust receipts, review queue operations, and audit
//! history. Backed by the shared core `TrustOpsStore`.

use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Json, Path, Query, Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Router,
};
use clap::Parser;
use pure_reason_core::{
    ensure_auth_configuration,
    pipeline::KantianPipeline,
    trust_ops::{OpsOverview, ReviewUpdate, TrustOpsStore, DEFAULT_LIST_LIMIT, MAX_LIST_LIMIT},
    ApiKeyRegistry, ApiPrincipal,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tracing::{error, info};

const MAX_RECORD_BYTES: usize = 70_000;
const AUTH_RATE_WINDOW_SECS: u64 = 60;
const AUTH_MAX_FAILURES_PER_WINDOW: u32 = 20;

#[derive(Parser)]
#[command(
    name = "pure-reason-dashboard",
    about = "PureReason trust operations console"
)]
struct Cli {
    #[arg(long, default_value = "127.0.0.1:8081")]
    bind: String,

    #[arg(long, default_value = "100")]
    max_history: usize,

    #[arg(long)]
    ops_dir: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    allow_unauthenticated: bool,
}

#[derive(Clone)]
struct AppState {
    store: Arc<TrustOpsStore>,
    pipeline: Arc<KantianPipeline>,
    history_limit: usize,
    api_keys: ApiKeyRegistry,
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

#[derive(Debug, Clone, Serialize)]
struct CurrentMetrics {
    total_processed: usize,
    window_size: usize,
    safe_count: usize,
    low_count: usize,
    medium_count: usize,
    high_count: usize,
    open_reviews: usize,
    blocked_count: usize,
    escalated_count: usize,
    average_ecs: f64,
    illusion_rate: f64,
    antinomy_rate: f64,
    paralogism_rate: f64,
    auto_regulation_rate: f64,
    epistemic_health_score: u32,
    generated_at: String,
}

#[derive(Debug, Clone, Serialize)]
struct HistoryPoint {
    timestamp: String,
    ecs: u8,
    risk_level: String,
}

#[derive(Debug, Clone, Serialize)]
struct HistoryResponse {
    points: Vec<HistoryPoint>,
    total_points: usize,
}

#[derive(Debug, Deserialize)]
struct RecordRequest {
    text: String,
    #[serde(default)]
    domain: String,
}

#[derive(Debug, Deserialize)]
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

fn default_limit() -> usize {
    DEFAULT_LIST_LIMIT
}

fn current_metrics(overview: &OpsOverview, issue_rates: (f64, f64, f64)) -> CurrentMetrics {
    let total = overview.total_receipts.max(1);
    let safe_rate = overview.safe_count as f64 / total as f64;
    let open_review_penalty = (overview.open_reviews as f64 / total as f64) * 12.0;
    let health = if overview.total_receipts == 0 {
        100
    } else {
        (overview.average_ecs + safe_rate * 18.0 - open_review_penalty)
            .clamp(0.0, 100.0)
            .round() as u32
    };

    CurrentMetrics {
        total_processed: overview.total_receipts,
        window_size: overview.total_receipts,
        safe_count: overview.safe_count,
        low_count: overview.low_count,
        medium_count: overview.medium_count,
        high_count: overview.high_count,
        open_reviews: overview.open_reviews,
        blocked_count: overview.blocked_count,
        escalated_count: overview.escalated_count,
        average_ecs: overview.average_ecs,
        illusion_rate: issue_rates.0,
        antinomy_rate: issue_rates.1,
        paralogism_rate: issue_rates.2,
        auto_regulation_rate: if overview.total_receipts == 0 {
            0.0
        } else {
            overview.auto_regulated_count as f64 / overview.total_receipts as f64
        },
        epistemic_health_score: health,
        generated_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    }
}

fn issue_rates(
    store: &TrustOpsStore,
    tenant: &str,
    limit: usize,
) -> pure_reason_core::Result<(f64, f64, f64)> {
    let receipts = store.list_receipts(Some(tenant), limit)?;
    if receipts.is_empty() {
        return Ok((0.0, 0.0, 0.0));
    }

    let mut illusions = 0usize;
    let mut antinomies = 0usize;
    let mut paralogisms = 0usize;
    for receipt in &receipts {
        let issues = &receipt.validation_certificate.issues;
        if issues.iter().any(|issue| issue.starts_with("Illusion:")) {
            illusions += 1;
        }
        if issues.iter().any(|issue| issue.starts_with("Antinomy:")) {
            antinomies += 1;
        }
        if issues.iter().any(|issue| issue.starts_with("Paralogism:")) {
            paralogisms += 1;
        }
    }

    let total = receipts.len() as f64;
    Ok((
        illusions as f64 / total,
        antinomies as f64 / total,
        paralogisms as f64 / total,
    ))
}

fn forbidden(message: &str) -> axum::response::Response {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": message })),
    )
        .into_response()
}

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

#[allow(clippy::result_large_err)]
fn to_json<T: Serialize>(value: &T) -> Result<serde_json::Value, axum::response::Response> {
    serde_json::to_value(value)
        .map_err(|error| internal_server_error("serialization failed", &error))
}

fn review_guard(principal: &ApiPrincipal) -> Option<axum::response::Response> {
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
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

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
    }
}

const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>PureReason · Trust Ops</title>
<script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.min.js" integrity="sha384-e6nUZLBkQ86NJ6TVVKAeSaK8jWa3NhkYWZFomE39AvDbQWeie9PlQqM3pmYW5d1g" crossorigin="anonymous"></script>
<style>
  :root {
    --bg: #0f1117; --card: #1a1d2e; --border: #2a2d3e;
    --text: #e2e8f0; --muted: #94a3b8; --accent: #6366f1;
    --safe: #22c55e; --low: #eab308; --medium: #f97316; --high: #ef4444;
  }
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body { background: var(--bg); color: var(--text); font-family: system-ui, sans-serif; padding: 1.5rem; }
  h1 { font-size: 1.6rem; font-weight: 700; color: var(--accent); }
  .subtitle { color: var(--muted); font-size: 0.92rem; margin: 0.35rem 0 1.5rem; }
  .auth-card { margin-bottom: 1rem; }
  .auth-row { display: flex; gap: 0.6rem; align-items: center; flex-wrap: wrap; margin-top: 0.85rem; }
  .auth-row input { flex: 1 1 280px; border: 1px solid var(--border); background: #111827; color: var(--text); padding: 0.55rem 0.7rem; border-radius: 0.55rem; }
  .auth-status { margin-top: 0.7rem; font-size: 0.85rem; color: var(--muted); }
  .auth-status.error { color: var(--high); }
  .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 1rem; margin-bottom: 1rem; }
  .card, .table-card, .chart-card { background: var(--card); border: 1px solid var(--border); border-radius: 0.85rem; padding: 1.1rem; }
  .kpi-label { font-size: 0.72rem; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; }
  .kpi-value { font-size: 2rem; font-weight: 700; margin-top: 0.25rem; }
  .charts { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin: 1rem 0; }
  .chart-card h3, .table-card h3 { font-size: 0.9rem; color: var(--muted); margin-bottom: 0.8rem; }
  .tables { display: grid; grid-template-columns: 1.35fr 1fr; gap: 1rem; margin-top: 1rem; }
  .tables-bottom { display: grid; grid-template-columns: 1fr; gap: 1rem; margin-top: 1rem; }
  table { width: 100%; border-collapse: collapse; font-size: 0.88rem; }
  th, td { text-align: left; padding: 0.55rem 0.4rem; border-bottom: 1px solid var(--border); vertical-align: top; }
  th { color: var(--muted); font-weight: 600; font-size: 0.78rem; text-transform: uppercase; letter-spacing: 0.04em; }
  .badge { display: inline-block; padding: 0.2rem 0.5rem; border-radius: 999px; font-size: 0.72rem; font-weight: 600; }
  .safe { color: var(--safe); }
  .low { color: var(--low); }
  .medium { color: var(--medium); }
  .high { color: var(--high); }
  .badge.safe { background: rgba(34,197,94,0.15); }
  .badge.low { background: rgba(234,179,8,0.15); }
  .badge.medium { background: rgba(249,115,22,0.15); }
  .badge.high { background: rgba(239,68,68,0.15); }
  .actions { display: flex; gap: 0.4rem; flex-wrap: wrap; }
  button { border: 1px solid var(--border); background: #111827; color: var(--text); padding: 0.35rem 0.6rem; border-radius: 0.55rem; cursor: pointer; }
  button:hover { border-color: var(--accent); }
  .muted { color: var(--muted); }
  .mono { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
  @media (max-width: 960px) {
    .charts, .tables { grid-template-columns: 1fr; }
  }
</style>
</head>
<body>
<h1>PureReason Trust Operations Console</h1>
<p class="subtitle">Persistent receipts, review queue, and audit history for epistemic safety workflows. Supply a bearer token when dashboard API auth is enabled.</p>

<div class="card auth-card">
  <div class="kpi-label">Dashboard API Token</div>
  <div class="auth-row">
    <input id="tokenInput" type="password" autocomplete="off" placeholder="Paste a local access token if dashboard auth is enabled">
    <button onclick="saveToken()">Save Token</button>
    <button onclick="clearToken()">Clear Token</button>
  </div>
  <div class="auth-status" id="authStatus">No token stored. Local unauthenticated mode still works when explicitly enabled on the server.</div>
</div>

<div class="grid">
  <div class="card"><div class="kpi-label">Health Score</div><div class="kpi-value" id="health">—</div></div>
  <div class="card"><div class="kpi-label">Total Receipts</div><div class="kpi-value" id="total">—</div></div>
  <div class="card"><div class="kpi-label">Open Reviews</div><div class="kpi-value" id="open_reviews">—</div></div>
  <div class="card"><div class="kpi-label">Average ECS</div><div class="kpi-value" id="avg_ecs">—</div></div>
  <div class="card"><div class="kpi-label">Auto-Regulated</div><div class="kpi-value" id="auto_reg">—</div></div>
</div>

<div class="charts">
  <div class="chart-card"><h3>Risk Distribution</h3><canvas id="riskChart"></canvas></div>
  <div class="chart-card"><h3>ECS Trend</h3><canvas id="ecsChart"></canvas></div>
</div>

<div class="tables">
  <div class="table-card">
    <h3>Recent Trust Receipts</h3>
    <table>
      <thead><tr><th>Time</th><th>ECS</th><th>Risk</th><th>Action</th><th>Domain</th><th>Preview</th></tr></thead>
      <tbody id="receiptsBody"><tr><td colspan="6" class="muted">Loading…</td></tr></tbody>
    </table>
  </div>
  <div class="table-card">
    <h3>Review Queue</h3>
    <table>
      <thead><tr><th>Status</th><th>Requested</th><th>Summary</th><th>Actions</th></tr></thead>
      <tbody id="reviewsBody"><tr><td colspan="4" class="muted">Loading…</td></tr></tbody>
    </table>
  </div>
</div>

<div class="tables-bottom">
  <div class="table-card">
    <h3>Audit Events</h3>
    <table>
      <thead><tr><th>Time</th><th>Kind</th><th>Message</th><th>Resource</th></tr></thead>
      <tbody id="auditBody"><tr><td colspan="4" class="muted">Loading…</td></tr></tbody>
    </table>
  </div>
</div>

<script>
let riskChart, ecsChart;
const TOKEN_STORAGE_KEY = 'pure-reason-dashboard-token';

function escapeHtml(value) {
  return String(value ?? '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;');
}

function riskClass(level) {
  const normalized = String(level || '').toLowerCase();
  if (normalized.includes('safe') || normalized.includes('allow') || normalized.includes('approved')) return 'safe';
  if (normalized.includes('low') || normalized.includes('warn') || normalized.includes('dismissed')) return 'low';
  if (normalized.includes('medium') || normalized.includes('regulate') || normalized.includes('rewrite')) return 'medium';
  return 'high';
}

function currentToken() {
  return sessionStorage.getItem(TOKEN_STORAGE_KEY) || '';
}

function authHeaders(headers = {}) {
  const resolved = new Headers(headers);
  const token = currentToken();
  if (token) {
    resolved.set('Authorization', `Bearer ${token}`);
  }
  return resolved;
}

function setAuthStatus(message, isError = false) {
  const node = document.getElementById('authStatus');
  node.textContent = message;
  node.classList.toggle('error', isError);
}

function saveToken() {
  const token = document.getElementById('tokenInput').value.trim();
  if (!token) {
    setAuthStatus('Enter a token before saving.', true);
    return;
  }
  sessionStorage.setItem(TOKEN_STORAGE_KEY, token);
  setAuthStatus('Bearer token saved for this browser session.');
  loadAll().catch(err => console.error(err));
}

function clearToken() {
  sessionStorage.removeItem(TOKEN_STORAGE_KEY);
  document.getElementById('tokenInput').value = '';
  setAuthStatus('Stored token cleared. Local unauthenticated mode can still work if the server allows it.');
  loadAll().catch(err => console.error(err));
}

async function fetchJson(url, options = {}) {
  const res = await fetch(url, {
    ...options,
    headers: authHeaders(options.headers || {})
  });
  if (!res.ok) throw new Error(`Request failed: ${url} (${res.status})`);
  return await res.json();
}

async function resolveReview(id, outcome) {
  const notes = window.prompt('Reviewer note (optional):', '') ?? '';
  const res = await fetch(`/api/reviews/${id}`, {
    method: 'POST',
    headers: authHeaders({ 'Content-Type': 'application/json' }),
    body: JSON.stringify({
      status: 'Resolved',
      outcome,
      notes: notes || null,
      corrected_text: null
    })
  });
  if (!res.ok) throw new Error(`Review update failed (${res.status})`);
  await loadAll();
}

function renderReceipts(receipts) {
  const tbody = document.getElementById('receiptsBody');
  if (!receipts.length) {
    tbody.innerHTML = '<tr><td colspan="6" class="muted">No receipts recorded yet.</td></tr>';
    return;
  }
  tbody.innerHTML = receipts.map(receipt => `
    <tr>
      <td class="mono">${escapeHtml(receipt.created_at.slice(11, 19))}</td>
      <td>${escapeHtml(receipt.ecs)}</td>
      <td><span class="badge ${riskClass(receipt.risk_level)}">${escapeHtml(receipt.risk_level)}</span></td>
      <td><span class="badge ${riskClass(receipt.policy_decision.action)}">${escapeHtml(receipt.policy_decision.action)}</span></td>
      <td>${escapeHtml(receipt.domain)}</td>
      <td>${escapeHtml(receipt.input_preview)}</td>
    </tr>
  `).join('');
}

function renderReviews(reviews) {
  const tbody = document.getElementById('reviewsBody');
  if (!reviews.length) {
    tbody.innerHTML = '<tr><td colspan="4" class="muted">Review queue is empty.</td></tr>';
    return;
  }
  tbody.innerHTML = reviews.map(review => {
    const resolved = review.status === 'Resolved';
    const actions = resolved ? '<span class="muted">resolved</span>' : `
      <div class="actions">
        <button onclick="resolveReview('${review.review_id}', 'Approved')">Approve</button>
        <button onclick="resolveReview('${review.review_id}', 'Rewritten')">Rewrite</button>
        <button onclick="resolveReview('${review.review_id}', 'Blocked')">Block</button>
        <button onclick="resolveReview('${review.review_id}', 'Dismissed')">Dismiss</button>
      </div>
    `;
    return `
      <tr>
        <td><span class="badge ${riskClass(review.status)}">${escapeHtml(review.status)}</span></td>
        <td><span class="badge ${riskClass(review.requested_action)}">${escapeHtml(review.requested_action)}</span></td>
        <td>${escapeHtml(review.summary)}</td>
        <td>${actions}</td>
      </tr>
    `;
  }).join('');
}

function renderAudit(events) {
  const tbody = document.getElementById('auditBody');
  if (!events.length) {
    tbody.innerHTML = '<tr><td colspan="4" class="muted">No audit events yet.</td></tr>';
    return;
  }
  tbody.innerHTML = events.map(event => `
    <tr>
      <td class="mono">${escapeHtml(event.timestamp.replace('T', ' ').slice(0, 19))}</td>
      <td>${escapeHtml(event.kind)}</td>
      <td>${escapeHtml(event.message)}</td>
      <td class="mono">${escapeHtml(event.resource_id)}</td>
    </tr>
  `).join('');
}

function renderCharts(metrics, history) {
  const riskData = [metrics.safe_count, metrics.low_count, metrics.medium_count, metrics.high_count];
  const historyLabels = history.map(point => point.timestamp.slice(11, 19));
  const ecsData = history.map(point => point.ecs);

  if (!riskChart) {
    riskChart = new Chart(document.getElementById('riskChart'), {
      type: 'doughnut',
      data: { labels: ['Safe', 'Low', 'Medium', 'High'], datasets: [{ data: riskData, backgroundColor: ['#22c55e','#eab308','#f97316','#ef4444'] }] },
      options: { plugins: { legend: { labels: { color: '#e2e8f0' } } } }
    });
    ecsChart = new Chart(document.getElementById('ecsChart'), {
      type: 'line',
      data: { labels: historyLabels, datasets: [{ data: ecsData, label: 'ECS', borderColor: '#6366f1', backgroundColor: 'rgba(99,102,241,0.2)', fill: true, tension: 0.25 }] },
      options: {
        plugins: { legend: { labels: { color: '#e2e8f0' } } },
        scales: {
          y: { min: 0, max: 100, ticks: { color: '#94a3b8' }, grid: { color: '#2a2d3e' } },
          x: { ticks: { color: '#94a3b8' }, grid: { color: '#2a2d3e' } }
        }
      }
    });
  } else {
    riskChart.data.datasets[0].data = riskData;
    riskChart.update();
    ecsChart.data.labels = historyLabels;
    ecsChart.data.datasets[0].data = ecsData;
    ecsChart.update();
  }
}

async function loadAll() {
  try {
    const [metrics, history, receipts, reviews, audit] = await Promise.all([
      fetchJson('/api/metrics/current'),
      fetchJson('/api/metrics/history'),
      fetchJson('/api/receipts?limit=12'),
      fetchJson('/api/reviews'),
      fetchJson('/api/audit?limit=20')
    ]);

    document.getElementById('health').textContent = `${metrics.epistemic_health_score}%`;
    document.getElementById('health').className = `kpi-value ${riskClass(metrics.epistemic_health_score < 50 ? 'high' : metrics.epistemic_health_score < 70 ? 'medium' : metrics.epistemic_health_score < 85 ? 'low' : 'safe')}`;
    document.getElementById('total').textContent = metrics.total_processed;
    document.getElementById('open_reviews').textContent = metrics.open_reviews;
    document.getElementById('avg_ecs').textContent = metrics.average_ecs.toFixed(1);
    document.getElementById('auto_reg').textContent = `${(metrics.auto_regulation_rate * 100).toFixed(1)}%`;

    renderCharts(metrics, history.points);
    renderReceipts(receipts);
    renderReviews(reviews);
    renderAudit(audit);
    if (currentToken()) {
      setAuthStatus('Connected with the stored bearer token.');
    } else {
      setAuthStatus('Connected without a token. This only works when the server is in explicit local unauthenticated mode.');
    }
  } catch (error) {
    if (String(error.message).includes('(401)')) {
      setAuthStatus('Dashboard API rejected the current request. Save a valid bearer token to continue.', true);
    } else {
      setAuthStatus(`Dashboard load failed: ${error.message}`, true);
    }
    throw error;
  }
}

document.getElementById('tokenInput').value = currentToken();
if (currentToken()) {
  setAuthStatus('Bearer token loaded from this browser session.');
}
loadAll().catch(err => console.error(err));
setInterval(() => loadAll().catch(err => console.error(err)), 10000);
</script>
</body>
</html>
"#;

async fn dashboard_html() -> impl IntoResponse {
    Html(DASHBOARD_HTML)
}

async fn api_overview(
    State(state): State<AppState>,
    Extension(principal): Extension<ApiPrincipal>,
) -> impl IntoResponse {
    let overview = match state
        .store
        .overview(Some(&principal.tenant), state.history_limit)
    {
        Ok(value) => value,
        Err(error) => return internal_server_error("failed to load dashboard overview", &error),
    };

    let rates = match issue_rates(&state.store, &principal.tenant, state.history_limit) {
        Ok(value) => value,
        Err(error) => return internal_server_error("failed to compute issue rates", &error),
    };

    match to_json(&current_metrics(&overview, rates)) {
        Ok(value) => (StatusCode::OK, Json(value)).into_response(),
        Err(error) => error,
    }
}

async fn api_history(
    State(state): State<AppState>,
    Extension(principal): Extension<ApiPrincipal>,
) -> impl IntoResponse {
    match state
        .store
        .overview(Some(&principal.tenant), state.history_limit)
    {
        Ok(overview) => {
            let points = overview
                .history
                .into_iter()
                .map(|point| HistoryPoint {
                    timestamp: point.timestamp,
                    ecs: point.ecs,
                    risk_level: point.risk_level,
                })
                .collect::<Vec<_>>();
            match to_json(&HistoryResponse {
                total_points: points.len(),
                points,
            }) {
                Ok(value) => (StatusCode::OK, Json(value)).into_response(),
                Err(error) => error,
            }
        }
        Err(error) => internal_server_error("failed to load dashboard history", &error),
    }
}

async fn api_receipts(
    State(state): State<AppState>,
    Extension(principal): Extension<ApiPrincipal>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    match state
        .store
        .list_receipts(Some(&principal.tenant), query.effective_limit())
    {
        Ok(receipts) => match to_json(&receipts) {
            Ok(value) => (StatusCode::OK, Json(value)).into_response(),
            Err(error) => error,
        },
        Err(error) => internal_server_error("failed to list dashboard receipts", &error),
    }
}

async fn api_reviews(
    State(state): State<AppState>,
    Extension(principal): Extension<ApiPrincipal>,
) -> impl IntoResponse {
    match state.store.list_reviews(Some(&principal.tenant)) {
        Ok(reviews) => match to_json(&reviews) {
            Ok(value) => (StatusCode::OK, Json(value)).into_response(),
            Err(error) => error,
        },
        Err(error) => internal_server_error("failed to list dashboard reviews", &error),
    }
}

async fn api_review_update(
    Path(review_id): Path<String>,
    State(state): State<AppState>,
    Extension(principal): Extension<ApiPrincipal>,
    Json(update): Json<ReviewUpdate>,
) -> impl IntoResponse {
    if let Some(response) = review_guard(&principal) {
        return response;
    }

    let actor_id = principal.actor_id();
    match state
        .store
        .update_review(&review_id, Some(&principal.tenant), &actor_id, update)
    {
        Ok(Some(review)) => match to_json(&review) {
            Ok(value) => (StatusCode::OK, Json(value)).into_response(),
            Err(error) => error,
        },
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Review item not found" })),
        )
            .into_response(),
        Err(error) => internal_server_error("failed to update dashboard review", &error),
    }
}

async fn api_audit(
    State(state): State<AppState>,
    Extension(principal): Extension<ApiPrincipal>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    match state
        .store
        .list_audit_events(Some(&principal.tenant), query.effective_limit())
    {
        Ok(events) => match to_json(&events) {
            Ok(value) => (StatusCode::OK, Json(value)).into_response(),
            Err(error) => error,
        },
        Err(error) => internal_server_error("failed to list dashboard audit events", &error),
    }
}

async fn metrics_current(
    State(state): State<AppState>,
    Extension(principal): Extension<ApiPrincipal>,
) -> impl IntoResponse {
    api_overview(State(state), Extension(principal)).await
}

async fn metrics_history(
    State(state): State<AppState>,
    Extension(principal): Extension<ApiPrincipal>,
) -> impl IntoResponse {
    api_history(State(state), Extension(principal)).await
}

async fn metrics_record(
    State(state): State<AppState>,
    Extension(principal): Extension<ApiPrincipal>,
    body: Bytes,
) -> impl IntoResponse {
    if let Some(response) = operator_guard(&principal) {
        return response;
    }
    if body.len() > MAX_RECORD_BYTES {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(serde_json::json!({
                "error": format!("Input exceeds maximum allowed size of {} bytes", MAX_RECORD_BYTES)
            })),
        )
            .into_response();
    }

    let request = match serde_json::from_slice::<RecordRequest>(&body) {
        Ok(value) => value,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid request body" })),
            )
                .into_response()
        }
    };

    let request_text = request.text.clone();
    let request_domain = request.domain.clone();
    let pipeline = Arc::clone(&state.pipeline);
    match tokio::task::spawn_blocking(move || pipeline.process(&request_text)).await {
        Ok(Ok(report)) => {
            let actor_id = principal.actor_id();
            match state.store.record_report(
                &report,
                Some(&principal.tenant),
                Some(&actor_id),
                if request_domain.is_empty() {
                    None
                } else {
                    Some(request_domain.as_str())
                },
            ) {
                Ok(evaluation) => Json(serde_json::json!({
                    "status": "recorded",
                    "receipt_id": evaluation.receipt.receipt_id,
                }))
                .into_response(),
                Err(error) => internal_server_error("failed to persist dashboard record", &error),
            }
        }
        Ok(Err(error)) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": error.to_string() })),
        )
            .into_response(),
        Err(error) => internal_server_error("dashboard record task panicked", &error),
    }
}

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
        "pure-reason-dashboard",
        &cli.bind,
        api_keys.auth_enabled,
        cli.allow_unauthenticated,
    ) {
        eprintln!("{error}");
        std::process::exit(2);
    }

    if !api_keys.auth_enabled {
        info!("Running pure-reason-dashboard without API-key auth; intended for local or explicitly trusted environments");
    }

    let store = match cli.ops_dir.clone() {
        Some(path) => TrustOpsStore::with_base(path),
        None => TrustOpsStore::new(),
    };
    let store = match store {
        Ok(store) => store,
        Err(error) => {
            eprintln!("failed to initialize trust ops store: {error}");
            std::process::exit(2);
        }
    };
    let state = AppState {
        store: Arc::new(store),
        pipeline: Arc::new(KantianPipeline::new()),
        history_limit: cli.max_history,
        api_keys,
        auth_rate_limiter: Arc::new(Mutex::new(AuthRateLimiter::new())),
    };

    let protected_routes = Router::new()
        .route("/api/overview", get(api_overview))
        .route("/api/receipts", get(api_receipts))
        .route("/api/reviews", get(api_reviews))
        .route("/api/reviews/:review_id", post(api_review_update))
        .route("/api/audit", get(api_audit))
        .route("/api/metrics/current", get(metrics_current))
        .route("/api/metrics/history", get(metrics_history))
        .route("/api/metrics/record", post(metrics_record))
        .layer(DefaultBodyLimit::max(MAX_RECORD_BYTES))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let app = Router::new()
        .route("/", get(dashboard_html))
        .merge(protected_routes)
        .with_state(state);

    let listener = match tokio::net::TcpListener::bind(&cli.bind).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("Failed to bind to {}: {}", cli.bind, error);
            std::process::exit(2);
        }
    };

    info!("PureReason Dashboard listening on http://{}", cli.bind);
    info!(
        "Open http://{} in your browser to view the dashboard",
        cli.bind
    );
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
