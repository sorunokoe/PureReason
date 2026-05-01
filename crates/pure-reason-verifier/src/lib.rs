//! # pure-reason-verifier
//!
//! Local-first synchronous verifier service for PureReason.
//!
//! Wraps the deterministic core modules (`KantianPipeline`,
//! `contradiction_detector`, `claims::annotate_claims`,
//! `StructuredDecisionValidator`) behind a clean, minimal public API
//! suitable for use by agent-facing entrypoints (MCP, CLI, loopback).
//!
//! ## Usage
//!
//! ```rust
//! use pure_reason_verifier::{VerifierService, VerificationRequest, ArtifactKind};
//!
//! let svc = VerifierService::new();
//! let req = VerificationRequest {
//!     content: "The sky is blue.".to_string(),
//!     kind: ArtifactKind::Text,
//!     trace_id: None,
//! };
//! let result = svc.verify(req).unwrap();
//! assert!(result.verdict.passed);
//! ```

pub mod triz_verifier; // TRIZ-enhanced verifier with all improvements

use pure_reason_core::{
    claims::annotate_claims,
    contradiction_detector::{extract_claims, find_contradictions},
    math_solver::MathSolver,
    pipeline::{KantianPipeline, RiskLevel},
    pre_verification_v2::{PreVerdict, PreVerificationConfig, PreVerifier},
    structured_validator::StructuredDecisionValidator,
};
use pure_reason_trace::{
    ids::TraceId,
    store::TraceStore,
    types::{TraceEvent, TraceEventPayload},
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum VerifierError {
    #[error("core pipeline error: {0}")]
    Pipeline(#[from] pure_reason_core::error::PureReasonError),
    #[error("trace error: {0}")]
    Trace(#[from] pure_reason_trace::error::TraceError),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, VerifierError>;

// ── Public types ──────────────────────────────────────────────────────────────

/// The kind of artifact being verified.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    /// Free-form text (assertion, explanation, report, …).
    Text,
    /// A structured JSON decision document.
    StructuredDecision,
}

/// Input to the verifier service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequest {
    /// The text or JSON content to verify.
    pub content: String,
    /// Whether `content` is plain text or a structured JSON decision.
    pub kind: ArtifactKind,
    /// Optional trace ID. When set, a `VerificationResult` trace event is
    /// appended to the default in-memory store attached to this service.
    pub trace_id: Option<String>,
}

/// Overall pass/fail verdict with a normalised risk score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verdict {
    /// `true` when no high-severity findings were detected.
    pub passed: bool,
    /// Normalised risk score in [0.0, 1.0]. Higher = more risky.
    pub risk_score: f64,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// Severity of an individual finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Category of an individual finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Contradiction,
    EpistemicOverreach,
    DomainViolation,
    StructuralAnomaly,
}

/// A single issue surfaced during verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    pub category: Category,
    /// Short, human-readable description of the finding.
    pub message: String,
}

/// The full output of a verification run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub verdict: Verdict,
    /// Zero or more individual issues.
    pub findings: Vec<Finding>,
    /// For `ArtifactKind::Text`: the regulated (rewritten) form when the
    /// pipeline rewrote content.  `None` when nothing was changed.
    pub regulated_text: Option<String>,
    /// Additional metadata (domain info, pre-verification stats, etc.)
    #[serde(default)]
    pub metadata: serde_json::Value,
}

// ── VerifierService ───────────────────────────────────────────────────────────

/// Synchronous, local-first verifier.
///
/// Holds stateless pipeline instances.  Construct once, call `verify` many
/// times.  All operations are pure-deterministic (no network, no API keys).
pub struct VerifierService {
    pipeline: KantianPipeline,
    structured_validator: StructuredDecisionValidator,
    /// Optional SQLite-backed trace store for emitting verification events.
    trace_store: Option<TraceStore>,
}

impl VerifierService {
    /// Create a new service with no trace persistence.
    pub fn new() -> Self {
        Self {
            pipeline: KantianPipeline::new(),
            structured_validator: StructuredDecisionValidator::new(),
            trace_store: None,
        }
    }

    /// Attach a trace store so that verification events are persisted when
    /// a `trace_id` is supplied in the request.
    pub fn with_trace_store(mut self, store: TraceStore) -> Self {
        self.trace_store = Some(store);
        self
    }

    /// Verify a single artifact and return structured findings.
    pub fn verify(&self, req: VerificationRequest) -> Result<VerificationResult> {
        let result = match req.kind {
            ArtifactKind::Text => self.verify_text(&req.content)?,
            ArtifactKind::StructuredDecision => self.verify_structured(&req.content)?,
        };

        if let Some(trace_id_str) = &req.trace_id {
            self.emit_trace(trace_id_str, &result)?;
        }

        Ok(result)
    }

    // ── Text verification ─────────────────────────────────────────────────────

    fn verify_text(&self, text: &str) -> Result<VerificationResult> {
        // Note: Pre-gate integration moved to triz_verifier.rs
        // This base service runs full pipeline
        self.verify_text_full_pipeline(text)
    }

    fn verify_text_full_pipeline(&self, text: &str) -> Result<VerificationResult> {
        let report = self.pipeline.process(text)?;
        let mut findings: Vec<Finding> = Vec::new();

        // ── Math verification pass ──
        if let Some(math_findings) = Self::check_arithmetic(text) {
            findings.extend(math_findings);
        }

        // ── Overconfidence detection pass ──
        if let Some(overconfidence_findings) = Self::check_overconfidence(text) {
            findings.extend(overconfidence_findings);
        }

        // ── Pipeline contradiction / antinomy pass ──
        if report.verdict.has_contradictions {
            findings.push(Finding {
                severity: Severity::Error,
                category: Category::Contradiction,
                message: "Antinomy detected: the text contains mutually contradictory claims about the same domain.".to_string(),
            });
        }

        // ── Contradiction detector pass ──
        let claims = extract_claims(text);
        if claims.len() >= 2 {
            let analysis = find_contradictions(&claims);
            for pair in &analysis.contradictions {
                let severity = if pair.confidence >= 0.85 {
                    Severity::Error
                } else {
                    Severity::Warning
                };
                findings.push(Finding {
                    severity,
                    category: Category::Contradiction,
                    message: pair.explanation.clone(),
                });
            }
        }

        // ── Claim annotation pass ──
        if let Ok(claim_report) = annotate_claims(text) {
            for annotation in &claim_report.claims {
                use pure_reason_core::claims::ClaimEvidenceStatus;
                if annotation.evidence.status == ClaimEvidenceStatus::Contradicted {
                    findings.push(Finding {
                        severity: Severity::Error,
                        category: Category::Contradiction,
                        message: format!("Claim '{}' is contradicted by evidence", annotation.text),
                    });
                }
            }
        }

        // ── Pipeline illusion / paralogism pass ──
        for illusion in &report.dialectic.illusions {
            use pure_reason_core::dialectic::IllusionSeverity;
            let severity = match illusion.severity {
                IllusionSeverity::Critical | IllusionSeverity::High => Severity::Error,
                IllusionSeverity::Medium => Severity::Warning,
                IllusionSeverity::Low => Severity::Info,
                _ => Severity::Info,
            };
            findings.push(Finding {
                severity,
                category: Category::EpistemicOverreach,
                message: format!(
                    "Transcendental illusion ({:?}): {}",
                    illusion.kind, illusion.description
                ),
            });
        }

        // ── Derive verdict ──
        let risk_score = risk_to_score(report.verdict.risk).max(findings_floor(&findings));
        let passed = report.verdict.risk < RiskLevel::High
            && !findings
                .iter()
                .any(|f| matches!(f.severity, Severity::Error | Severity::Critical));

        let summary = build_text_summary(passed, report.verdict.risk, &findings);

        let regulated_text = if report.regulated_text != text {
            Some(report.regulated_text)
        } else {
            None
        };

        Ok(VerificationResult {
            verdict: Verdict {
                passed,
                risk_score,
                summary,
            },
            findings,
            regulated_text,
            metadata: serde_json::json!({}),
        })
    }

    // ── Structured decision verification ─────────────────────────────────────

    fn verify_structured(&self, json_str: &str) -> Result<VerificationResult> {
        let validation = self
            .structured_validator
            .validate_json(json_str)
            .map_err(VerifierError::Pipeline)?;

        let mut findings: Vec<Finding> = Vec::new();

        for c in &validation.internal_contradictions {
            let severity = if c.severity == "CRITICAL" {
                Severity::Critical
            } else if c.severity == "HIGH" {
                Severity::Error
            } else {
                Severity::Warning
            };
            findings.push(Finding {
                severity,
                category: Category::Contradiction,
                message: c.explanation.clone(),
            });
        }

        for issue in &validation.epistemic_issues {
            findings.push(Finding {
                severity: Severity::Warning,
                category: Category::EpistemicOverreach,
                message: format!(
                    "Field '{}' contains epistemic overreach: {}",
                    issue.field_path,
                    issue.issues.join("; ")
                ),
            });
        }

        for violation in &validation.domain_violations {
            findings.push(Finding {
                severity: Severity::Error,
                category: Category::DomainViolation,
                message: format!(
                    "Domain constraint '{}' violated at '{}': {}",
                    violation.constraint_id, violation.field_path, violation.message
                ),
            });
        }

        let risk_score = overall_risk_to_score(&validation.overall_risk);
        let passed = !findings
            .iter()
            .any(|f| matches!(f.severity, Severity::Critical | Severity::Error));

        let summary = validation.summary.clone();

        Ok(VerificationResult {
            verdict: Verdict {
                passed,
                risk_score,
                summary,
            },
            findings,
            regulated_text: None,
            metadata: serde_json::json!({}),
        })
    }

    // ── Arithmetic verification ───────────────────────────────────────────────

    fn check_arithmetic(text: &str) -> Option<Vec<Finding>> {
        use regex::Regex;

        // Pattern: "X divided by Y equals Z" or "X / Y = Z" or "X + Y = Z"
        let patterns = vec![
            // "120 divided by 2 equals 90"
            Regex::new(r"(\d+(?:\.\d+)?)\s+divided\s+by\s+(\d+(?:\.\d+)?)\s+(?:equals?|is)\s+(\d+(?:\.\d+)?)").ok(),
            // "120 / 2 = 90"
            Regex::new(r"(\d+(?:\.\d+)?)\s*/\s*(\d+(?:\.\d+)?)\s*=\s*(\d+(?:\.\d+)?)").ok(),
            // "120 plus 30 equals 90" or "120 + 30 = 90"
            Regex::new(r"(\d+(?:\.\d+)?)\s*(?:\+|plus)\s*(\d+(?:\.\d+)?)\s*(?:=|equals?|is)\s*(\d+(?:\.\d+)?)").ok(),
            // "120 minus 30 equals 90" or "120 - 30 = 90"
            Regex::new(r"(\d+(?:\.\d+)?)\s*(?:-|minus)\s*(\d+(?:\.\d+)?)\s*(?:=|equals?|is)\s*(\d+(?:\.\d+)?)").ok(),
            // "120 times 2 equals 90" or "120 * 2 = 90"
            Regex::new(r"(\d+(?:\.\d+)?)\s*(?:\*|×|times)\s*(\d+(?:\.\d+)?)\s*(?:=|equals?|is)\s*(\d+(?:\.\d+)?)").ok(),
        ];

        let mut findings = Vec::new();

        for (idx, maybe_pattern) in patterns.into_iter().enumerate() {
            let pattern = match maybe_pattern {
                Some(p) => p,
                None => continue,
            };

            for cap in pattern.captures_iter(text) {
                let left_num: f64 = cap[1].parse().ok()?;
                let right_num: f64 = cap[2].parse().ok()?;
                let claimed: f64 = cap[3].parse().ok()?;

                let (operator, correct) = match idx {
                    0 | 1 => ("/", left_num / right_num), // division
                    2 => ("+", left_num + right_num),     // addition
                    3 => ("-", left_num - right_num),     // subtraction
                    4 => ("*", left_num * right_num),     // multiplication
                    _ => continue,
                };

                let expr = format!("{} {} {}", left_num, operator, right_num);
                let claim = MathSolver::verify_claim(&expr, &claimed.to_string(), claimed);

                if !claim.is_correct && claim.relative_error > 0.1 {
                    let severity = if claim.relative_error > 10.0 {
                        Severity::Error
                    } else {
                        Severity::Warning
                    };

                    findings.push(Finding {
                        severity,
                        category: Category::EpistemicOverreach,
                        message: format!(
                            "Arithmetic error: {} = {:.2}, not {:.2} (error: {:.1}%)",
                            expr, correct, claimed, claim.relative_error
                        ),
                    });
                }
            }
        }

        if findings.is_empty() {
            None
        } else {
            Some(findings)
        }
    }

    // ── Overconfidence detection ──────────────────────────────────────────────

    fn check_overconfidence(text: &str) -> Option<Vec<Finding>> {
        use regex::Regex;

        let text_lower = text.to_lowercase();
        let mut findings = Vec::new();

        // Critical certainty markers (especially problematic in medical/finance/legal)
        let critical_patterns = vec![
            ("must have", "absolute certainty without evidence"),
            (
                "definitely has",
                "definitive diagnosis without qualification",
            ),
            ("certainly is", "unqualified certainty"),
            ("absolutely will", "future certainty claim"),
            ("guaranteed to", "inappropriate guarantee"),
            ("proven fact that", "unqualified fact claim"),
        ];

        for (pattern, reason) in critical_patterns {
            if text_lower.contains(pattern) {
                findings.push(Finding {
                    severity: Severity::Warning,
                    category: Category::EpistemicOverreach,
                    message: format!(
                        "Overconfidence detected: '{}' indicates {}. Consider hedging language.",
                        pattern, reason
                    ),
                });
            }
        }

        // Check for high-confidence medical/financial claims
        let medical_pattern = Regex::new(r"\b(?:patient|diagnosis|treatment|prognosis)\b").ok()?;
        let financial_pattern =
            Regex::new(r"\b(?:stock|investment|return|profit|portfolio)\b").ok()?;

        let is_medical = medical_pattern.is_match(&text_lower);
        let is_financial = financial_pattern.is_match(&text_lower);

        if is_medical || is_financial {
            let domain = if is_medical { "medical" } else { "financial" };

            // Domain-specific certainty markers
            let high_certainty_markers = vec!["will", "always", "never", "impossible", "certain"];

            for marker in high_certainty_markers {
                if text_lower.contains(marker) {
                    findings.push(Finding {
                        severity: Severity::Warning,
                        category: Category::DomainViolation,
                        message: format!(
                            "High-certainty language ('{}') in {} domain requires explicit evidence and hedging.",
                            marker, domain
                        ),
                    });
                    break; // Only report once per domain
                }
            }
        }

        if findings.is_empty() {
            None
        } else {
            Some(findings)
        }
    }

    // ── Trace emission ────────────────────────────────────────────────────────

    fn emit_trace(&self, trace_id_str: &str, result: &VerificationResult) -> Result<()> {
        let store = match &self.trace_store {
            Some(s) => s,
            None => {
                tracing::debug!("trace_id supplied but no TraceStore attached; skipping emit");
                return Ok(());
            }
        };

        let trace_id = TraceId::from_str(trace_id_str)
            .map_err(|e| VerifierError::InvalidInput(format!("invalid trace_id: {e}")))?;

        let payload = TraceEventPayload::VerificationResult {
            passed: result.verdict.passed,
            score: 1.0 - result.verdict.risk_score,
            details: result.verdict.summary.clone(),
        };

        let event = TraceEvent::new(trace_id, None, payload);
        store.append(&event)?;

        Ok(())
    }
}

impl Default for VerifierService {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn risk_to_score(risk: RiskLevel) -> f64 {
    match risk {
        RiskLevel::Safe => 0.0,
        RiskLevel::Low => 0.25,
        RiskLevel::Medium => 0.60,
        RiskLevel::High => 0.90,
        _ => 1.0,
    }
}

fn overall_risk_to_score(risk: &str) -> f64 {
    match risk {
        "SAFE" => 0.0,
        "LOW" => 0.25,
        "MEDIUM" => 0.60,
        "HIGH" => 0.80,
        "CRITICAL" => 1.0,
        _ => 0.50,
    }
}

fn findings_floor(findings: &[Finding]) -> f64 {
    if findings
        .iter()
        .any(|f| matches!(f.severity, Severity::Critical))
    {
        1.0
    } else if findings
        .iter()
        .any(|f| matches!(f.severity, Severity::Error))
    {
        0.8
    } else if findings
        .iter()
        .any(|f| matches!(f.severity, Severity::Warning))
    {
        0.5
    } else {
        0.0
    }
}

fn build_text_summary(passed: bool, risk: RiskLevel, findings: &[Finding]) -> String {
    if findings.is_empty() && passed {
        return "No issues detected. Content is epistemically sound.".to_string();
    }
    let errors = findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Error | Severity::Critical))
        .count();
    let warnings = findings
        .iter()
        .filter(|f| f.severity == Severity::Warning)
        .count();
    format!(
        "Risk: {}. {} error(s), {} warning(s) detected.",
        risk, errors, warnings
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use pure_reason_trace::store::TraceStore;

    fn svc() -> VerifierService {
        VerifierService::new()
    }

    #[test]
    fn safe_text_passes() {
        let result = svc()
            .verify(VerificationRequest {
                content: "Water boils at 100 degrees Celsius at sea level.".to_string(),
                kind: ArtifactKind::Text,
                trace_id: None,
            })
            .unwrap();

        assert!(result.verdict.passed, "safe text should pass");
        assert!(
            result.verdict.risk_score < 0.5,
            "risk score should be low for safe text"
        );
        assert!(
            result
                .findings
                .iter()
                .all(|f| f.severity != Severity::Critical),
            "no critical findings for safe text"
        );
    }

    #[test]
    fn direct_contradiction_detected() {
        // Classic cosmological antinomy — triggers pipeline contradiction detection.
        let text = "The universe had a beginning in time. \
                    The universe has no beginning and is eternal.";

        let result = svc()
            .verify(VerificationRequest {
                content: text.to_string(),
                kind: ArtifactKind::Text,
                trace_id: None,
            })
            .unwrap();

        let has_contradiction = result
            .findings
            .iter()
            .any(|f| f.category == Category::Contradiction);
        assert!(
            has_contradiction,
            "direct contradiction should be detected; findings: {:?}",
            result.findings
        );
        assert!(
            !result.verdict.passed,
            "contradictory text should not pass verification"
        );
    }

    #[test]
    fn structured_decision_violation_detected() {
        // Medical: warfarin is both contraindicated and prescribed.
        let json = r#"{
            "contraindications": ["warfarin"],
            "prescribed": ["aspirin", "warfarin"]
        }"#;

        let result = svc()
            .verify(VerificationRequest {
                content: json.to_string(),
                kind: ArtifactKind::StructuredDecision,
                trace_id: None,
            })
            .unwrap();

        assert!(
            !result.verdict.passed,
            "critical medical contradiction should fail"
        );
        let has_contradiction = result
            .findings
            .iter()
            .any(|f| f.category == Category::Contradiction && f.severity == Severity::Critical);
        assert!(
            has_contradiction,
            "should have a critical contradiction finding; findings: {:?}",
            result.findings
        );
    }

    #[test]
    fn trace_event_emitted_when_store_attached() {
        let store = TraceStore::open_in_memory().unwrap();
        let svc = VerifierService::new().with_trace_store(store);

        let trace_id = pure_reason_trace::ids::TraceId::new();

        let result = svc
            .verify(VerificationRequest {
                content: "The sun rises in the east.".to_string(),
                kind: ArtifactKind::Text,
                trace_id: Some(trace_id.to_string()),
            })
            .unwrap();

        // The store is moved into the service; query via service's store field.
        let events = svc
            .trace_store
            .as_ref()
            .unwrap()
            .list_by_trace(&trace_id, 10)
            .unwrap();

        assert_eq!(events.len(), 1, "one trace event should be appended");
        assert!(
            matches!(
                &events[0].payload,
                pure_reason_trace::types::TraceEventPayload::VerificationResult {
                    passed,
                    ..
                } if *passed == result.verdict.passed
            ),
            "trace payload should match verdict"
        );
    }
}
