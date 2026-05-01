//! # Epistemic SLA Platform (S-IV-7)
//!
//! Defines SLA targets for epistemic quality and monitors AI deployments continuously.
//!
//! The SLA is defined in TOML and loaded at server startup. The monitoring loop
//! updates metrics on every /analyze API call and evaluates SLA compliance.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use pure_reason_core::pipeline::{PipelineReport, RiskLevel};

// ─── SLA Definition ──────────────────────────────────────────────────────────

/// A single SLA target metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaTarget {
    /// Metric name: "high_risk_rate" | "antinomy_rate" | "overreach_rate"
    pub metric: String,
    /// Maximum acceptable value (as fraction, e.g., 0.02 = 2%)
    pub max_value: f64,
    /// Action on breach: "alert" | "log"
    pub breach_action: String,
}

/// SLA definition for an AI deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaDefinition {
    pub name: String,
    /// Rolling window in seconds for metric calculation.
    pub window_seconds: u64,
    pub targets: Vec<SlaTarget>,
}

impl Default for SlaDefinition {
    fn default() -> Self {
        Self {
            name: "Default Epistemic SLA".to_string(),
            window_seconds: 86400, // 24h
            targets: vec![
                SlaTarget {
                    metric: "high_risk_rate".to_string(),
                    max_value: 0.05,
                    breach_action: "alert".to_string(),
                },
                SlaTarget {
                    metric: "antinomy_rate".to_string(),
                    max_value: 0.02,
                    breach_action: "alert".to_string(),
                },
                SlaTarget {
                    metric: "overreach_rate".to_string(),
                    max_value: 0.03,
                    breach_action: "alert".to_string(),
                },
            ],
        }
    }
}

// ─── Metrics record ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct MetricRecord {
    timestamp: i64,
    risk_level: RiskLevel,
    has_antinomy: bool,
    has_overreach: bool,
    auto_regulated: bool,
}

impl MetricRecord {
    fn from_report(report: &PipelineReport) -> Self {
        let has_overreach = report.dialectic.illusions.iter().any(|i| {
            matches!(
                i.kind,
                pure_reason_core::dialectic::IllusionKind::EpistemicOverreach
            )
        });
        Self {
            timestamp: Utc::now().timestamp(),
            risk_level: report.verdict.risk,
            has_antinomy: report.verdict.has_contradictions,
            has_overreach,
            auto_regulated: !report.transformations.is_empty(),
        }
    }
}

// ─── SlaMonitor ──────────────────────────────────────────────────────────────

/// Monitors epistemic quality metrics and evaluates SLA compliance.
#[derive(Clone)]
pub struct SlaMonitor {
    definition: SlaDefinition,
    records: Arc<Mutex<VecDeque<MetricRecord>>>,
}

impl SlaMonitor {
    pub fn new(definition: SlaDefinition) -> Self {
        Self {
            definition,
            records: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Record a pipeline report.
    pub fn record(&self, report: &PipelineReport) {
        let record = MetricRecord::from_report(report);
        // Recover from a poisoned mutex rather than panicking in a service path.
        let mut records = match self.records.lock() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        records.push_back(record);
        // Trim old records outside the window
        let cutoff = Utc::now().timestamp() - self.definition.window_seconds as i64;
        while records
            .front()
            .map(|r| r.timestamp < cutoff)
            .unwrap_or(false)
        {
            records.pop_front();
        }
        // Cap at 100,000 records to prevent unbounded memory
        while records.len() > 100_000 {
            records.pop_front();
        }
    }

    /// Generate the current SLA report.
    pub fn report(&self) -> SlaReport {
        let records = match self.records.lock() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        let total = records.len();

        if total == 0 {
            return SlaReport {
                sla_name: self.definition.name.clone(),
                total_requests: 0,
                window_seconds: self.definition.window_seconds,
                target_results: Vec::new(),
                overall_compliant: true,
                epistemic_health_score: 100,
                auto_regulated_count: 0,
                generated_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            };
        }

        let high_risk_count = records
            .iter()
            .filter(|r| r.risk_level >= RiskLevel::High)
            .count();
        let antinomy_count = records.iter().filter(|r| r.has_antinomy).count();
        let overreach_count = records.iter().filter(|r| r.has_overreach).count();
        let regulated_count = records.iter().filter(|r| r.auto_regulated).count();

        let high_risk_rate = high_risk_count as f64 / total as f64;
        let antinomy_rate = antinomy_count as f64 / total as f64;
        let overreach_rate = overreach_count as f64 / total as f64;

        let mut target_results = Vec::new();
        let mut overall_compliant = true;

        for target in &self.definition.targets {
            let actual = match target.metric.as_str() {
                "high_risk_rate" => high_risk_rate,
                "antinomy_rate" => antinomy_rate,
                "overreach_rate" => overreach_rate,
                _ => 0.0,
            };
            let met = actual <= target.max_value;
            if !met {
                overall_compliant = false;
            }
            target_results.push(SlaTargetResult {
                metric: target.metric.clone(),
                target_max: target.max_value,
                actual,
                met,
            });
        }

        // Epistemic health score: 100 - weighted penalty
        let health = (100.0
            - (high_risk_rate * 50.0 + antinomy_rate * 30.0 + overreach_rate * 20.0) * 100.0)
            .clamp(0.0, 100.0) as u32;

        SlaReport {
            sla_name: self.definition.name.clone(),
            total_requests: total,
            window_seconds: self.definition.window_seconds,
            target_results,
            overall_compliant,
            epistemic_health_score: health,
            auto_regulated_count: regulated_count,
            generated_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        }
    }
}

// ─── SlaReport ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaTargetResult {
    pub metric: String,
    pub target_max: f64,
    pub actual: f64,
    pub met: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaReport {
    pub sla_name: String,
    pub total_requests: usize,
    pub window_seconds: u64,
    pub target_results: Vec<SlaTargetResult>,
    pub overall_compliant: bool,
    pub epistemic_health_score: u32,
    /// Number of outputs that were automatically rewritten to regulative language.
    /// This is a factual operational metric. Economic impact assessment is out of scope.
    pub auto_regulated_count: usize,
    pub generated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pure_reason_core::pipeline::KantianPipeline;

    #[test]
    fn empty_monitor_is_compliant() {
        let monitor = SlaMonitor::new(SlaDefinition::default());
        let report = monitor.report();
        assert!(report.overall_compliant);
        assert_eq!(report.total_requests, 0);
    }

    #[test]
    fn monitor_records_and_reports() {
        let monitor = SlaMonitor::new(SlaDefinition::default());
        let pipeline = KantianPipeline::new();
        let report = pipeline.process("Water boils at 100 degrees.").unwrap();
        monitor.record(&report);
        let sla_report = monitor.report();
        assert_eq!(sla_report.total_requests, 1);
        assert!(sla_report.overall_compliant); // one safe request is compliant
    }
}
