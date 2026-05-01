//! Performance Monitoring: Track latency, throughput, and resource utilization
//!
//! TRIZ Principle: Feedback + Inspection
//! Make invisible signals visible for performance optimization and debugging.
//!
//! This module provides observability into the reasoning pipeline without adding
//! overhead. Metrics are collected via tracing spans and can be exported to monitoring
//! systems like Prometheus, Datadog, or custom backends.

use std::time::Instant;
use tracing::debug;
use tracing::info;

/// Performance metrics for a single reasoning operation
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Total latency in milliseconds
    pub latency_ms: f64,
    /// Phase A (deterministic heuristics) latency
    pub phase_a_ms: f64,
    /// Phase B (model inference) latency
    pub phase_b_ms: f64,
    /// Phase C (self-audit) latency
    pub phase_c_ms: f64,
    /// Phase D (contradiction detection) latency
    pub phase_d_ms: f64,
    /// Memory used in bytes
    pub memory_bytes: u64,
    /// Throughput: operations per second
    pub throughput_ops_per_sec: f64,
}

impl PerformanceMetrics {
    /// Create a new performance metrics instance from individual phase timings
    pub fn new(
        phase_a_ms: f64,
        phase_b_ms: f64,
        phase_c_ms: f64,
        phase_d_ms: f64,
        memory_bytes: u64,
    ) -> Self {
        let latency_ms = phase_a_ms + phase_b_ms + phase_c_ms + phase_d_ms;
        let throughput_ops_per_sec = if latency_ms > 0.0 {
            1000.0 / latency_ms
        } else {
            0.0
        };

        Self {
            latency_ms,
            phase_a_ms,
            phase_b_ms,
            phase_c_ms,
            phase_d_ms,
            memory_bytes,
            throughput_ops_per_sec,
        }
    }

    /// Summary string for logging
    pub fn summary(&self) -> String {
        format!(
            "latency={}ms (A:{}ms B:{}ms C:{}ms D:{}ms) throughput={:.1} ops/sec memory={} bytes",
            self.latency_ms as i64,
            self.phase_a_ms as i64,
            self.phase_b_ms as i64,
            self.phase_c_ms as i64,
            self.phase_d_ms as i64,
            self.throughput_ops_per_sec,
            self.memory_bytes,
        )
    }
}

/// Timer for measuring phase execution
pub struct PhaseTimer {
    start: Instant,
    phase_name: &'static str,
}

impl PhaseTimer {
    /// Create a new timer for a phase
    pub fn new(phase_name: &'static str) -> Self {
        debug!("Starting phase: {}", phase_name);
        Self {
            start: Instant::now(),
            phase_name,
        }
    }

    /// Finish timing and return elapsed milliseconds
    pub fn finish(self) -> f64 {
        let elapsed = self.start.elapsed().as_secs_f64() * 1000.0;
        info!("{} completed in {:.2}ms", self.phase_name, elapsed);
        elapsed
    }
}

/// Latency histogram for analyzing performance distribution
#[derive(Debug, Clone)]
pub struct LatencyHistogram {
    /// Buckets: (upper_bound_ms, count)
    buckets: Vec<(f64, u64)>,
    /// Total samples
    total_samples: u64,
    /// Minimum latency
    min_ms: f64,
    /// Maximum latency
    max_ms: f64,
    /// Sum of all latencies
    sum_ms: f64,
}

impl LatencyHistogram {
    /// Create a new histogram with standard buckets (1ms, 5ms, 10ms, 25ms, 50ms, 100ms, 250ms, 500ms, ∞)
    pub fn new() -> Self {
        Self {
            buckets: vec![
                (1.0, 0),
                (5.0, 0),
                (10.0, 0),
                (25.0, 0),
                (50.0, 0),
                (100.0, 0),
                (250.0, 0),
                (500.0, 0),
                (f64::INFINITY, 0),
            ],
            total_samples: 0,
            min_ms: f64::INFINITY,
            max_ms: 0.0,
            sum_ms: 0.0,
        }
    }

    /// Record a latency observation
    pub fn record(&mut self, latency_ms: f64) {
        self.total_samples += 1;
        self.min_ms = self.min_ms.min(latency_ms);
        self.max_ms = self.max_ms.max(latency_ms);
        self.sum_ms += latency_ms;

        for (bound, count) in &mut self.buckets {
            if latency_ms <= *bound {
                *count += 1;
                break;
            }
        }
    }

    /// Get percentile latency
    pub fn percentile(&self, p: f64) -> f64 {
        if self.total_samples == 0 {
            return 0.0;
        }

        let target_count = (self.total_samples as f64 * (p / 100.0)) as u64;
        let mut cumulative = 0u64;

        for (_, count) in &self.buckets {
            cumulative += count;
            if cumulative >= target_count {
                return self.average();
            }
        }

        self.max_ms
    }

    /// Get average latency
    pub fn average(&self) -> f64 {
        if self.total_samples == 0 {
            return 0.0;
        }
        self.sum_ms / self.total_samples as f64
    }

    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "samples={} avg={:.2}ms min={:.2}ms max={:.2}ms p50={:.2}ms p95={:.2}ms p99={:.2}ms",
            self.total_samples,
            self.average(),
            self.min_ms,
            self.max_ms,
            self.percentile(50.0),
            self.percentile(95.0),
            self.percentile(99.0),
        )
    }
}

impl Default for LatencyHistogram {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics_creation() {
        let metrics = PerformanceMetrics::new(10.0, 50.0, 20.0, 15.0, 1024);
        assert_eq!(metrics.latency_ms, 95.0);
        assert_eq!(metrics.phase_a_ms, 10.0);
        assert!(metrics.throughput_ops_per_sec > 0.0);
    }

    #[test]
    fn test_performance_metrics_summary() {
        let metrics = PerformanceMetrics::new(10.0, 50.0, 20.0, 15.0, 1024);
        let summary = metrics.summary();
        assert!(summary.contains("latency=95ms"));
        assert!(summary.contains("ops/sec"));
    }

    #[test]
    fn test_latency_histogram_recording() {
        let mut hist = LatencyHistogram::new();
        hist.record(5.0);
        hist.record(15.0);
        hist.record(50.0);
        assert_eq!(hist.total_samples, 3);
        assert_eq!(hist.min_ms, 5.0);
        assert_eq!(hist.max_ms, 50.0);
    }

    #[test]
    fn test_latency_histogram_average() {
        let mut hist = LatencyHistogram::new();
        hist.record(10.0);
        hist.record(20.0);
        hist.record(30.0);
        let avg = hist.average();
        assert!((avg - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_latency_histogram_summary() {
        let mut hist = LatencyHistogram::new();
        for i in 1..=100 {
            hist.record(i as f64);
        }
        let summary = hist.summary();
        assert!(summary.contains("samples=100"));
        assert!(summary.contains("avg="));
    }

    #[test]
    fn test_phase_timer() {
        let timer = PhaseTimer::new("test_phase");
        let elapsed = timer.finish();
        assert!(elapsed >= 0.0);
    }

    #[test]
    fn test_performance_metrics_zero_latency() {
        let metrics = PerformanceMetrics::new(0.0, 0.0, 0.0, 0.0, 0);
        assert_eq!(metrics.throughput_ops_per_sec, 0.0);
    }
}
