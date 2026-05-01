//! # Trust Operations Layer (TRIZ S-IX)
//!
//! Turns a [`PipelineReport`](crate::pipeline::PipelineReport) into operational
//! trust artifacts: receipts, policy decisions, review items, and audit events.
//!
//! TRIZ framing:
//! - **Segmentation**: shared core logic; thin API/dashboard adapters.
//! - **Universality**: one receipt powers review, dashboard, export, and audit.
//! - **Preliminary action**: generate policy/receipt at analysis time.
//! - **Feedback**: review actions create durable audit history.
//!
//! ## Storage backend (J2 — SQLite)
//!
//! Prior implementation used append-only JSONL files + a global `Mutex<()>`,
//! which required a full O(n) file scan for every read. The current
//! implementation uses a single SQLite database (`trust_ops.db`) with indexed
//! columns for tenant filtering and time-ordered queries:
//!
//! - `receipts(receipt_id PK, tenant, created_at, ...)` — indexed on
//!   `(tenant, created_at DESC)` for O(log n) list operations.
//! - `reviews(review_id PK, tenant, status, updated_at, ...)` — indexed on
//!   `(tenant, status)`.
//! - `audit_events(event_id PK, tenant, timestamp, ...)` — indexed on
//!   `(tenant, timestamp DESC)`.
//!
//! Full JSON blobs are stored in the `data` column so the schema never needs
//! an application-level migration when new fields are added to the structs.
//!
//! On first open, if legacy `.jsonl` / `reviews.json` files exist they are
//! imported into SQLite and renamed to `*.imported`.

use crate::{
    calibration::PipelineCalibration,
    certificate::ValidationCertificate,
    claims::ClaimEvidenceStatus,
    domain::{builtin_profile, ConstraintViolation},
    error::Result,
    pipeline::{PipelineReport, RiskLevel},
};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use uuid::Uuid;

pub const DEFAULT_LIST_LIMIT: usize = 50;
pub const MAX_LIST_LIMIT: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum TrustRole {
    Viewer,
    #[default]
    Operator,
    Reviewer,
    Admin,
}

impl TrustRole {
    pub fn can_review(self) -> bool {
        matches!(self, Self::Reviewer | Self::Admin)
    }

    pub fn can_operate(self) -> bool {
        matches!(self, Self::Operator | Self::Reviewer | Self::Admin)
    }
}

impl std::fmt::Display for TrustRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Viewer => write!(f, "viewer"),
            Self::Operator => write!(f, "operator"),
            Self::Reviewer => write!(f, "reviewer"),
            Self::Admin => write!(f, "admin"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PolicyAction {
    Allow,
    Warn,
    Regulate,
    Escalate,
    Block,
}

impl std::fmt::Display for PolicyAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allow => write!(f, "allow"),
            Self::Warn => write!(f, "warn"),
            Self::Regulate => write!(f, "regulate"),
            Self::Escalate => write!(f, "escalate"),
            Self::Block => write!(f, "block"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub policy_id: String,
    pub domain: String,
    pub action: PolicyAction,
    pub review_required: bool,
    pub reasons: Vec<String>,
    pub triggered_constraints: Vec<ConstraintViolation>,
    pub regulations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskyClaimSummary {
    pub claim_id: String,
    pub text: String,
    pub risk: String,
    pub evidence_status: String,
    pub support_score: f64,
    pub contradiction_score: f64,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustCounts {
    pub total_claims: usize,
    pub risky_claims: usize,
    pub contradicted_claims: usize,
    pub novel_claims: usize,
    pub unresolved_claims: usize,
    pub missing_context_claims: usize,
    pub transformations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustReceipt {
    pub receipt_id: String,
    pub created_at: String,
    pub tenant: String,
    pub domain: String,
    pub input_hash: String,
    pub input_preview: String,
    pub ecs: u8,
    pub risk_level: String,
    pub dominant_category: Option<String>,
    pub primary_language_game: Option<String>,
    pub summary: String,
    pub regulated_text: Option<String>,
    pub policy_decision: PolicyDecision,
    pub counts: TrustCounts,
    pub risky_claims: Vec<RiskyClaimSummary>,
    pub validation_certificate: ValidationCertificate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewStatus {
    Open,
    InReview,
    Resolved,
}

impl std::fmt::Display for ReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::InReview => write!(f, "in_review"),
            Self::Resolved => write!(f, "resolved"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewOutcome {
    Approved,
    Rewritten,
    Blocked,
    Dismissed,
}

impl std::fmt::Display for ReviewOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Approved => write!(f, "approved"),
            Self::Rewritten => write!(f, "rewritten"),
            Self::Blocked => write!(f, "blocked"),
            Self::Dismissed => write!(f, "dismissed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResolution {
    pub outcome: ReviewOutcome,
    pub reviewer: String,
    pub notes: Option<String>,
    pub corrected_text: Option<String>,
    pub resolved_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewItem {
    pub review_id: String,
    pub receipt_id: String,
    pub tenant: String,
    pub status: ReviewStatus,
    pub requested_action: PolicyAction,
    pub risk_level: String,
    pub summary: String,
    pub created_at: String,
    pub updated_at: String,
    pub resolution: Option<ReviewResolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewUpdate {
    pub status: ReviewStatus,
    pub outcome: Option<ReviewOutcome>,
    pub notes: Option<String>,
    pub corrected_text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventKind {
    ReceiptRecorded,
    ReviewQueued,
    ReviewUpdated,
    Exported,
    WebhookSent,
    WebhookFailed,
}

impl std::fmt::Display for AuditEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReceiptRecorded => write!(f, "receipt_recorded"),
            Self::ReviewQueued => write!(f, "review_queued"),
            Self::ReviewUpdated => write!(f, "review_updated"),
            Self::Exported => write!(f, "exported"),
            Self::WebhookSent => write!(f, "webhook_sent"),
            Self::WebhookFailed => write!(f, "webhook_failed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub timestamp: String,
    pub tenant: String,
    pub actor: String,
    pub kind: AuditEventKind,
    pub resource_id: String,
    pub message: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpsHistoryPoint {
    pub timestamp: String,
    pub ecs: u8,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpsOverview {
    pub total_receipts: usize,
    pub open_reviews: usize,
    pub safe_count: usize,
    pub low_count: usize,
    pub medium_count: usize,
    pub high_count: usize,
    pub average_ecs: f64,
    pub blocked_count: usize,
    pub escalated_count: usize,
    pub auto_regulated_count: usize,
    pub latest_receipt_at: Option<String>,
    pub history: Vec<OpsHistoryPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpsExportBundle {
    pub overview: OpsOverview,
    pub receipts: Vec<TrustReceipt>,
    pub reviews: Vec<ReviewItem>,
    pub audit_events: Vec<AuditEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEvaluation {
    pub receipt: TrustReceipt,
    pub review_item: Option<ReviewItem>,
    pub audit_events: Vec<AuditEvent>,
}

impl PolicyDecision {
    pub fn from_report(report: &PipelineReport, domain: Option<&str>) -> Self {
        let profile = builtin_profile(domain.unwrap_or("general"));
        let dominant_category = report.verdict.dominant_category.as_deref();
        let category_score = dominant_category
            .map(|name| category_confidence(report, name))
            .unwrap_or(report.verdict.pre_score)
            .max(if dominant_category.is_some() {
                0.75
            } else {
                0.0
            });
        let triggered_constraints =
            profile.check_all(&report.input, dominant_category, category_score);

        let mut action = PolicyAction::Allow;
        let mut reasons = Vec::new();

        if report.verdict.risk == RiskLevel::Medium {
            action = PolicyAction::Warn;
            reasons.push("pipeline reported medium epistemic risk".to_string());
        }

        if report.verdict.risk == RiskLevel::High {
            action = PolicyAction::Escalate;
            reasons.push("pipeline reported high epistemic risk".to_string());
        }

        if !report.transformations.is_empty() {
            action = action.max(PolicyAction::Regulate);
            reasons.push("regulative transformation is available".to_string());
        }

        if report.claim_analysis.contradicted_count > 0 || report.verdict.has_contradictions {
            action = action.max(PolicyAction::Escalate);
            reasons.push("contradicted claims require human review".to_string());
        }

        if report.claim_analysis.novel_count > 0 && action < PolicyAction::Warn {
            action = PolicyAction::Warn;
            reasons.push("novel claims should be surfaced to the operator".to_string());
        }

        for violation in &triggered_constraints {
            let mapped = match violation.action.as_str() {
                "block" => PolicyAction::Block,
                "regulate" => PolicyAction::Regulate,
                "warn" => PolicyAction::Warn,
                _ => PolicyAction::Warn,
            };
            action = action.max(mapped);
            reasons.push(violation.message.clone());
        }

        let review_required = matches!(action, PolicyAction::Escalate | PolicyAction::Block)
            || report.claim_analysis.contradicted_count > 0
            || (!triggered_constraints.is_empty() && action != PolicyAction::Allow);

        let mut regulations = Vec::new();
        if let Some(primary) = profile.primary_regulation {
            regulations.push(primary);
        }
        regulations.extend(profile.secondary_regulations);
        for violation in &triggered_constraints {
            if let Some(reference) = &violation.regulation_reference {
                if !regulations.contains(reference) {
                    regulations.push(reference.clone());
                }
            }
        }

        Self {
            policy_id: format!(
                "{}-trust-policy",
                profile.name.to_lowercase().replace(' ', "-")
            ),
            domain: profile.name,
            action,
            review_required,
            reasons,
            triggered_constraints,
            regulations,
        }
    }
}

impl TrustReceipt {
    pub fn from_report(
        report: &PipelineReport,
        tenant: Option<&str>,
        domain: Option<&str>,
    ) -> Self {
        let policy_decision = PolicyDecision::from_report(report, domain);
        let validation_certificate = ValidationCertificate::from_report(report);
        let risky_claims = report
            .claim_analysis
            .claims
            .iter()
            .filter(|claim| {
                claim.risk != RiskLevel::Safe
                    || claim.evidence.status == ClaimEvidenceStatus::Contradicted
            })
            .map(|claim| RiskyClaimSummary {
                claim_id: claim.claim_id.clone(),
                text: claim.text.clone(),
                risk: claim.risk.to_string(),
                evidence_status: claim.evidence.status.to_string(),
                support_score: claim.evidence.support_score,
                contradiction_score: claim.evidence.contradiction_score,
                issues: claim
                    .illusion_issues
                    .iter()
                    .chain(claim.antinomy_issues.iter())
                    .chain(claim.paralogism_issues.iter())
                    .cloned()
                    .collect(),
            })
            .collect();

        let regulated_text =
            if !report.transformations.is_empty() && report.regulated_text != report.input {
                Some(report.regulated_text.clone())
            } else {
                None
            };

        Self {
            receipt_id: Uuid::new_v4().to_string(),
            created_at: now_rfc3339(),
            tenant: tenant.unwrap_or("local").to_string(),
            domain: policy_decision.domain.clone(),
            input_hash: validation_certificate.content_hash.clone(),
            input_preview: preview(&report.input),
            ecs: report.ecs(),
            risk_level: report.verdict.risk.to_string(),
            dominant_category: report.verdict.dominant_category.clone(),
            primary_language_game: report.verdict.primary_language_game.clone(),
            summary: report.summary.clone(),
            regulated_text,
            policy_decision,
            counts: TrustCounts {
                total_claims: report.claim_analysis.claims.len(),
                risky_claims: report.claim_analysis.risky_count,
                contradicted_claims: report.claim_analysis.contradicted_count,
                novel_claims: report.claim_analysis.novel_count,
                unresolved_claims: report.claim_analysis.unresolved_count,
                missing_context_claims: report.claim_analysis.missing_context_count,
                transformations: report.transformations.len(),
            },
            risky_claims,
            validation_certificate,
        }
    }
}

impl ReviewItem {
    pub fn from_receipt(receipt: &TrustReceipt, _actor: &str) -> Self {
        let timestamp = now_rfc3339();
        Self {
            review_id: Uuid::new_v4().to_string(),
            receipt_id: receipt.receipt_id.clone(),
            tenant: receipt.tenant.clone(),
            status: ReviewStatus::Open,
            requested_action: receipt.policy_decision.action,
            risk_level: receipt.risk_level.clone(),
            summary: receipt.summary.clone(),
            created_at: timestamp.clone(),
            updated_at: timestamp,
            resolution: None,
        }
    }
}

impl AuditEvent {
    pub fn new(
        tenant: impl Into<String>,
        actor: impl Into<String>,
        kind: AuditEventKind,
        resource_id: impl Into<String>,
        message: impl Into<String>,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            timestamp: now_rfc3339(),
            tenant: tenant.into(),
            actor: actor.into(),
            kind,
            resource_id: resource_id.into(),
            message: message.into(),
            metadata,
        }
    }
}

pub fn evaluate_report(
    report: &PipelineReport,
    tenant: Option<&str>,
    actor: Option<&str>,
    domain: Option<&str>,
) -> TrustEvaluation {
    let actor = actor.unwrap_or("system");
    let receipt = TrustReceipt::from_report(report, tenant, domain);
    let mut audit_events = vec![AuditEvent::new(
        receipt.tenant.clone(),
        actor,
        AuditEventKind::ReceiptRecorded,
        receipt.receipt_id.clone(),
        "Trust receipt recorded",
        serde_json::json!({
            "ecs": receipt.ecs,
            "risk_level": receipt.risk_level,
            "domain": receipt.domain,
            "action": receipt.policy_decision.action,
        }),
    )];

    let review_item = if receipt.policy_decision.review_required {
        let review = ReviewItem::from_receipt(&receipt, actor);
        audit_events.push(AuditEvent::new(
            receipt.tenant.clone(),
            actor,
            AuditEventKind::ReviewQueued,
            review.review_id.clone(),
            "Review item queued",
            serde_json::json!({
                "receipt_id": receipt.receipt_id,
                "requested_action": review.requested_action,
            }),
        ));
        Some(review)
    } else {
        None
    };

    TrustEvaluation {
        receipt,
        review_item,
        audit_events,
    }
}

pub struct TrustOpsStore {
    base: PathBuf,
    db_path: PathBuf,
}

impl TrustOpsStore {
    pub fn new() -> Result<Self> {
        Self::open(default_ops_dir())
    }

    pub fn with_base(base: impl Into<PathBuf>) -> Result<Self> {
        Self::open(base.into())
    }

    fn open(base: PathBuf) -> Result<Self> {
        fs::create_dir_all(&base)?;
        let db_path = base.join("trust_ops.db");
        let conn = Connection::open(&db_path).map_err(|e| std::io::Error::other(e.to_string()))?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;",
        )
        .map_err(|e| std::io::Error::other(e.to_string()))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS receipts (
                receipt_id   TEXT PRIMARY KEY,
                tenant       TEXT NOT NULL,
                created_at   TEXT NOT NULL,
                risk_level   TEXT NOT NULL DEFAULT 'SAFE',
                ecs          INTEGER NOT NULL DEFAULT 0,
                policy_action TEXT NOT NULL DEFAULT 'allow',
                has_regulated INTEGER NOT NULL DEFAULT 0,
                data         TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_receipts_tenant_time
                ON receipts(tenant, created_at DESC);

            CREATE TABLE IF NOT EXISTS reviews (
                review_id    TEXT PRIMARY KEY,
                tenant       TEXT NOT NULL,
                status       TEXT NOT NULL DEFAULT 'Open',
                updated_at   TEXT NOT NULL,
                data         TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_reviews_tenant_status
                ON reviews(tenant, status);

            CREATE TABLE IF NOT EXISTS audit_events (
                event_id     TEXT PRIMARY KEY,
                tenant       TEXT NOT NULL,
                actor        TEXT NOT NULL,
                kind         TEXT NOT NULL,
                timestamp    TEXT NOT NULL,
                data         TEXT NOT NULL
            );
             CREATE INDEX IF NOT EXISTS idx_audit_tenant_time
                 ON audit_events(tenant, timestamp DESC);",
        )
        .map_err(|e| std::io::Error::other(e.to_string()))?;

        let store = Self { base, db_path };
        store.migrate_legacy_jsonl();
        Ok(store)
    }

    fn connect(&self) -> Result<Connection> {
        let conn =
            Connection::open(&self.db_path).map_err(|e| std::io::Error::other(e.to_string()))?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;",
        )
        .map_err(|e| std::io::Error::other(e.to_string()))?;
        Ok(conn)
    }

    /// One-shot import of legacy JSONL data (best-effort; errors are silently skipped).
    /// Query data blobs from any table, optionally filtering by tenant and limiting rows.
    ///
    /// Uses `(?1 IS NULL OR tenant = ?1)` so a single prepared statement handles both cases.
    fn query_blobs_conn(
        &self,
        conn: &Connection,
        table: &str,
        order_by: &str,
        tenant: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<String>> {
        let lim = limit.map(|l| l as i64).unwrap_or(i64::MAX);
        let sql = format!(
            "SELECT data FROM {table} WHERE (?1 IS NULL OR tenant = ?1) ORDER BY {order_by} LIMIT ?2"
        );
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        let rows = stmt
            .query_map(params![tenant, lim], |row| row.get(0))
            .map_err(|e| std::io::Error::other(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    fn migrate_legacy_jsonl(&self) {
        let Ok(conn) = self.connect() else {
            return;
        };

        // receipts.jsonl
        let receipts_path = self.base.join("receipts.jsonl");
        if receipts_path.exists() {
            if let Ok(content) = fs::read_to_string(&receipts_path) {
                for line in content.lines().filter(|l| !l.trim().is_empty()) {
                    if let Ok(r) = serde_json::from_str::<TrustReceipt>(line) {
                        let _ = conn.execute(
                            "INSERT OR IGNORE INTO receipts
                             (receipt_id, tenant, created_at, risk_level, ecs,
                              policy_action, has_regulated, data)
                             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                            params![
                                r.receipt_id,
                                r.tenant,
                                r.created_at,
                                r.risk_level,
                                r.ecs as i64,
                                r.policy_decision.action.to_string(),
                                r.regulated_text.is_some() as i32,
                                line,
                            ],
                        );
                    }
                }
            }
            let _ = fs::rename(&receipts_path, self.base.join("receipts.jsonl.imported"));
        }

        // reviews.json (full JSON array)
        let reviews_path = self.base.join("reviews.json");
        if reviews_path.exists() {
            if let Ok(content) = fs::read_to_string(&reviews_path) {
                if let Ok(reviews) = serde_json::from_str::<Vec<ReviewItem>>(&content) {
                    for r in reviews {
                        let blob = serde_json::to_string(&r).unwrap_or_default();
                        let _ = conn.execute(
                            "INSERT OR IGNORE INTO reviews
                             (review_id, tenant, status, updated_at, data)
                             VALUES (?1,?2,?3,?4,?5)",
                            params![
                                r.review_id,
                                r.tenant,
                                r.status.to_string(),
                                r.updated_at,
                                blob,
                            ],
                        );
                    }
                }
            }
            let _ = fs::rename(&reviews_path, self.base.join("reviews.json.imported"));
        }

        // audit.jsonl
        let audit_path = self.base.join("audit.jsonl");
        if audit_path.exists() {
            if let Ok(content) = fs::read_to_string(&audit_path) {
                for line in content.lines().filter(|l| !l.trim().is_empty()) {
                    if let Ok(e) = serde_json::from_str::<AuditEvent>(line) {
                        let _ = conn.execute(
                            "INSERT OR IGNORE INTO audit_events
                             (event_id, tenant, actor, kind, timestamp, data)
                             VALUES (?1,?2,?3,?4,?5,?6)",
                            params![
                                e.event_id,
                                e.tenant,
                                e.actor,
                                e.kind.to_string(),
                                e.timestamp,
                                line,
                            ],
                        );
                    }
                }
            }
            let _ = fs::rename(&audit_path, self.base.join("audit.jsonl.imported"));
        }
    }

    pub fn base(&self) -> &PathBuf {
        &self.base
    }

    pub fn record_report(
        &self,
        report: &PipelineReport,
        tenant: Option<&str>,
        actor: Option<&str>,
        domain: Option<&str>,
    ) -> Result<TrustEvaluation> {
        let evaluation = evaluate_report(report, tenant, actor, domain);
        self.persist_evaluation(&evaluation)?;
        Ok(evaluation)
    }

    pub fn persist_evaluation(&self, evaluation: &TrustEvaluation) -> Result<()> {
        let conn = self.connect()?;
        let r = &evaluation.receipt;
        let blob = serde_json::to_string(r)?;
        conn.execute(
            "INSERT OR IGNORE INTO receipts
             (receipt_id, tenant, created_at, risk_level, ecs,
              policy_action, has_regulated, data)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                r.receipt_id,
                r.tenant,
                r.created_at,
                r.risk_level,
                r.ecs as i64,
                r.policy_decision.action.to_string(),
                r.regulated_text.is_some() as i32,
                blob,
            ],
        )
        .map_err(|e| std::io::Error::other(e.to_string()))?;

        if let Some(review) = &evaluation.review_item {
            let blob = serde_json::to_string(review)?;
            conn.execute(
                "INSERT OR IGNORE INTO reviews
                 (review_id, tenant, status, updated_at, data)
                 VALUES (?1,?2,?3,?4,?5)",
                params![
                    review.review_id,
                    review.tenant,
                    review.status.to_string(),
                    review.updated_at,
                    blob,
                ],
            )
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        }

        for event in &evaluation.audit_events {
            self.insert_audit_event_conn(&conn, event)?;
        }

        Ok(())
    }

    fn insert_audit_event_conn(&self, conn: &Connection, event: &AuditEvent) -> Result<()> {
        let blob = serde_json::to_string(event)?;
        conn.execute(
            "INSERT OR IGNORE INTO audit_events
             (event_id, tenant, actor, kind, timestamp, data)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                event.event_id,
                event.tenant,
                event.actor,
                event.kind.to_string(),
                event.timestamp,
                blob,
            ],
        )
        .map_err(|e| std::io::Error::other(e.to_string()))?;
        Ok(())
    }

    pub fn list_receipts(&self, tenant: Option<&str>, limit: usize) -> Result<Vec<TrustReceipt>> {
        let limit = effective_limit(limit);
        let conn = self.connect()?;
        let rows =
            self.query_blobs_conn(&conn, "receipts", "created_at DESC", tenant, Some(limit))?;
        rows.iter()
            .map(|blob| serde_json::from_str::<TrustReceipt>(blob).map_err(Into::into))
            .collect()
    }

    pub fn get_receipt(
        &self,
        receipt_id: &str,
        tenant: Option<&str>,
    ) -> Result<Option<TrustReceipt>> {
        let conn = self.connect()?;
        let blob: Option<String> = if let Some(t) = tenant {
            conn.query_row(
                "SELECT data FROM receipts WHERE receipt_id = ?1 AND tenant = ?2 LIMIT 1",
                params![receipt_id, t],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| std::io::Error::other(e.to_string()))?
        } else {
            conn.query_row(
                "SELECT data FROM receipts WHERE receipt_id = ?1 LIMIT 1",
                params![receipt_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| std::io::Error::other(e.to_string()))?
        };
        blob.map(|b| serde_json::from_str::<TrustReceipt>(&b).map_err(Into::into))
            .transpose()
    }

    pub fn list_reviews(&self, tenant: Option<&str>) -> Result<Vec<ReviewItem>> {
        let conn = self.connect()?;
        let rows = self.query_blobs_conn(&conn, "reviews", "updated_at DESC", tenant, None)?;
        rows.iter()
            .map(|blob| serde_json::from_str::<ReviewItem>(blob).map_err(Into::into))
            .collect()
    }

    pub fn update_review(
        &self,
        review_id: &str,
        tenant: Option<&str>,
        actor: &str,
        update: ReviewUpdate,
    ) -> Result<Option<ReviewItem>> {
        let conn = self.connect()?;

        // Load the review
        let (sql, p): (&str, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(t) = tenant {
            (
                "SELECT data FROM reviews WHERE review_id = ?1 AND tenant = ?2 LIMIT 1",
                vec![Box::new(review_id.to_string()), Box::new(t.to_string())],
            )
        } else {
            (
                "SELECT data FROM reviews WHERE review_id = ?1 LIMIT 1",
                vec![Box::new(review_id.to_string())],
            )
        };
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        let refs: Vec<&dyn rusqlite::ToSql> = p.iter().map(|x| x.as_ref()).collect();
        let blob = stmt
            .query_row(refs.as_slice(), |row| row.get::<_, String>(0))
            .optional()
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let Some(blob) = blob else {
            return Ok(None);
        };
        let mut review: ReviewItem = serde_json::from_str(&blob)?;

        review.status = update.status;
        review.updated_at = now_rfc3339();
        review.resolution = if review.status == ReviewStatus::Resolved {
            Some(ReviewResolution {
                outcome: update.outcome.unwrap_or(ReviewOutcome::Approved),
                reviewer: actor.to_string(),
                notes: update.notes,
                corrected_text: update.corrected_text,
                resolved_at: now_rfc3339(),
            })
        } else {
            None
        };

        let updated_blob = serde_json::to_string(&review)?;
        conn.execute(
            "UPDATE reviews SET status = ?1, updated_at = ?2, data = ?3
              WHERE review_id = ?4",
            params![
                review.status.to_string(),
                review.updated_at,
                updated_blob,
                review.review_id,
            ],
        )
        .map_err(|e| std::io::Error::other(e.to_string()))?;

        let audit = AuditEvent::new(
            review.tenant.clone(),
            actor,
            AuditEventKind::ReviewUpdated,
            review.review_id.clone(),
            "Review item updated",
            serde_json::json!({
                "status": review.status,
                "outcome": review.resolution.as_ref().map(|r| r.outcome),
            }),
        );
        self.insert_audit_event_conn(&conn, &audit)?;

        Ok(Some(review))
    }

    pub fn list_audit_events(&self, tenant: Option<&str>, limit: usize) -> Result<Vec<AuditEvent>> {
        let limit = effective_limit(limit);
        let conn = self.connect()?;
        let rows =
            self.query_blobs_conn(&conn, "audit_events", "timestamp DESC", tenant, Some(limit))?;
        rows.iter()
            .map(|blob| serde_json::from_str::<AuditEvent>(blob).map_err(Into::into))
            .collect()
    }

    pub fn append_audit_event(&self, event: &AuditEvent) -> Result<()> {
        let conn = self.connect()?;
        self.insert_audit_event_conn(&conn, event)
    }

    pub fn export_bundle(&self, tenant: Option<&str>, limit: usize) -> Result<OpsExportBundle> {
        let receipts = self.list_receipts(tenant, limit)?;
        let reviews = self.list_reviews(tenant)?;
        let audit_events = self.list_audit_events(tenant, limit)?;
        let overview = self.overview(tenant, limit)?;

        if let Some(t) = tenant {
            self.append_audit_event(&AuditEvent::new(
                t,
                "system",
                AuditEventKind::Exported,
                "bundle",
                "Trust operations export generated",
                serde_json::json!({ "receipt_count": receipts.len() }),
            ))?;
        }

        Ok(OpsExportBundle {
            overview,
            receipts,
            reviews,
            audit_events,
        })
    }

    pub fn overview(&self, tenant: Option<&str>, limit: usize) -> Result<OpsOverview> {
        let receipts = self.list_receipts(tenant, limit)?;
        let reviews = self.list_reviews(tenant)?;
        let total = receipts.len();
        let safe_count = receipts.iter().filter(|r| r.risk_level == "SAFE").count();
        let low_count = receipts.iter().filter(|r| r.risk_level == "LOW").count();
        let medium_count = receipts.iter().filter(|r| r.risk_level == "MEDIUM").count();
        let high_count = receipts.iter().filter(|r| r.risk_level == "HIGH").count();
        let blocked_count = receipts
            .iter()
            .filter(|r| r.policy_decision.action == PolicyAction::Block)
            .count();
        let escalated_count = receipts
            .iter()
            .filter(|r| r.policy_decision.action == PolicyAction::Escalate)
            .count();
        let auto_regulated_count = receipts
            .iter()
            .filter(|r| r.regulated_text.is_some())
            .count();
        let average_ecs = if total == 0 {
            0.0
        } else {
            receipts.iter().map(|r| r.ecs as f64).sum::<f64>() / total as f64
        };
        let history = receipts
            .iter()
            .rev()
            .take(50)
            .map(|r| OpsHistoryPoint {
                timestamp: r.created_at.clone(),
                ecs: r.ecs,
                risk_level: r.risk_level.clone(),
            })
            .collect();

        Ok(OpsOverview {
            total_receipts: total,
            open_reviews: reviews
                .iter()
                .filter(|r| r.status != ReviewStatus::Resolved)
                .count(),
            safe_count,
            low_count,
            medium_count,
            high_count,
            average_ecs,
            blocked_count,
            escalated_count,
            auto_regulated_count,
            latest_receipt_at: receipts.first().map(|r| r.created_at.clone()),
            history,
        })
    }
}

pub fn default_ops_dir() -> PathBuf {
    home_dir().join(".pure-reason").join("ops")
}

fn category_confidence(report: &PipelineReport, name: &str) -> f64 {
    report
        .understanding
        .category_analysis
        .applications
        .iter()
        .find(|application| application.category.name().eq_ignore_ascii_case(name))
        .map(|application| application.confidence.value())
        .unwrap_or(report.verdict.pre_score)
}

fn preview(text: &str) -> String {
    let preview: String = text.chars().take(120).collect();
    if text.chars().count() > 120 {
        format!("{preview}...")
    } else {
        preview
    }
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn effective_limit(requested: usize) -> usize {
    if requested == 0 {
        DEFAULT_LIST_LIMIT
    } else {
        requested.min(MAX_LIST_LIMIT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::KantianPipeline;
    use tempfile::TempDir;

    #[test]
    fn high_risk_report_creates_review_item() {
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process("The universe had a beginning in time. The universe has no beginning.")
            .unwrap();
        let evaluation = evaluate_report(&report, Some("tenant-a"), Some("alice"), Some("legal"));
        assert_eq!(evaluation.receipt.tenant, "tenant-a");
        assert!(evaluation.receipt.policy_decision.review_required);
        assert!(evaluation.review_item.is_some());
    }

    #[test]
    fn medical_certainty_claim_is_regulated_and_queued_for_review() {
        let pipeline = KantianPipeline::new();
        let report = pipeline.process("The patient must have cancer.").unwrap();

        let evaluation = evaluate_report(&report, Some("tenant-a"), Some("alice"), Some("medical"));

        assert_eq!(
            evaluation.receipt.policy_decision.action,
            PolicyAction::Regulate
        );
        assert!(evaluation.receipt.policy_decision.review_required);
        assert_eq!(
            evaluation
                .receipt
                .policy_decision
                .triggered_constraints
                .iter()
                .map(|constraint| constraint.constraint_id.as_str())
                .collect::<Vec<_>>(),
            vec!["no-certain-diagnoses"]
        );
        assert!(evaluation.review_item.is_some());
    }

    #[test]
    fn store_roundtrip_receipts_and_reviews() {
        let dir = TempDir::new().unwrap();
        let store = TrustOpsStore::with_base(dir.path()).unwrap();
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process("The universe had a beginning in time. The universe has no beginning.")
            .unwrap();

        let evaluation = store
            .record_report(&report, Some("tenant-a"), Some("alice"), Some("legal"))
            .unwrap();

        let receipts = store.list_receipts(Some("tenant-a"), 10).unwrap();
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].receipt_id, evaluation.receipt.receipt_id);

        let reviews = store.list_reviews(Some("tenant-a")).unwrap();
        assert_eq!(reviews.len(), 1);

        let updated = store
            .update_review(
                &reviews[0].review_id,
                Some("tenant-a"),
                "reviewer-1",
                ReviewUpdate {
                    status: ReviewStatus::Resolved,
                    outcome: Some(ReviewOutcome::Blocked),
                    notes: Some("Contradiction confirmed".to_string()),
                    corrected_text: None,
                },
            )
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, ReviewStatus::Resolved);

        let audit = store.list_audit_events(Some("tenant-a"), 10).unwrap();
        assert!(audit.len() >= 3);
    }
}
