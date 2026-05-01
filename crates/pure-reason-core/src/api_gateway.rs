//! API Gateway: Rate limiting, health checks, and request routing
//!
//! TRIZ Principle: Preliminary Action + Self-Service
//! Proactively protect the system with rate limiting and health monitoring.
//!
//! This module provides enterprise-grade API gateway features:
//! - Rate limiting (token bucket algorithm)
//! - Health checks (readiness, liveness, detailed diagnostics)
//! - Request throttling and backpressure
//! - Graceful degradation during overload

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

/// Request rate limiter using token bucket algorithm
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Maximum tokens (requests) allowed per window
    capacity: u64,
    /// Current number of available tokens
    tokens: Arc<AtomicU64>,
    /// Refill rate: tokens per second
    refill_per_sec: f64,
    /// Last refill timestamp
    last_refill: Arc<AtomicU64>,
}

impl RateLimiter {
    /// Create a new rate limiter
    /// * `capacity` - Maximum burst requests
    /// * `refill_per_sec` - Requests allowed per second (e.g., 1000 = 1000 req/sec)
    pub fn new(capacity: u64, refill_per_sec: f64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            capacity,
            tokens: Arc::new(AtomicU64::new(capacity)),
            refill_per_sec,
            last_refill: Arc::new(AtomicU64::new(now)),
        }
    }

    /// Try to acquire a token; returns true if allowed, false if rate limit exceeded
    pub fn allow_request(&self) -> bool {
        self.refill_tokens();
        let current = self.tokens.load(Ordering::SeqCst);
        if current > 0 {
            self.tokens.store(current - 1, Ordering::SeqCst);
            true
        } else {
            warn!("Rate limit exceeded");
            false
        }
    }

    /// Get current available tokens
    pub fn available_tokens(&self) -> u64 {
        self.refill_tokens();
        self.tokens.load(Ordering::SeqCst)
    }

    /// Refill tokens based on elapsed time
    fn refill_tokens(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let last = self.last_refill.load(Ordering::SeqCst);

        if now > last {
            let elapsed = (now - last) as f64;
            let new_tokens = (elapsed * self.refill_per_sec) as u64;
            let current = self.tokens.load(Ordering::SeqCst);
            let refilled = std::cmp::min(current + new_tokens, self.capacity);
            self.tokens.store(refilled, Ordering::SeqCst);
            self.last_refill.store(now, Ordering::SeqCst);
        }
    }
}

/// Health check status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// System is healthy and ready
    Healthy,
    /// System is degraded but operational
    Degraded,
    /// System is not ready
    Unhealthy,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

/// Health check diagnostics
#[derive(Debug, Clone)]
pub struct HealthCheck {
    /// Overall status
    pub status: HealthStatus,
    /// Uptime in seconds
    pub uptime_secs: u64,
    /// Memory usage estimate
    pub memory_mb: u64,
    /// Test requests processed
    pub test_requests: u64,
    /// Request errors in last minute
    pub error_rate_percent: f64,
    /// Detailed message
    pub message: String,
}

impl HealthCheck {
    /// Create a new health check
    pub fn new(status: HealthStatus, uptime_secs: u64) -> Self {
        Self {
            status,
            uptime_secs,
            memory_mb: 0,
            test_requests: 0,
            error_rate_percent: 0.0,
            message: "OK".to_string(),
        }
    }

    /// Check if system is ready (liveness probe)
    pub fn is_alive(&self) -> bool {
        self.status != HealthStatus::Unhealthy
    }

    /// Check if system is ready to accept requests (readiness probe)
    pub fn is_ready(&self) -> bool {
        self.status == HealthStatus::Healthy
    }

    /// Get status code for HTTP response (200 = healthy, 503 = unhealthy)
    pub fn http_status_code(&self) -> u16 {
        match self.status {
            HealthStatus::Healthy => 200,
            HealthStatus::Degraded => 200, // Still accepting requests
            HealthStatus::Unhealthy => 503,
        }
    }
}

/// API Gateway for managing requests
pub struct ApiGateway {
    /// Rate limiter
    rate_limiter: RateLimiter,
    /// Start time for uptime calculation
    start_time: SystemTime,
    /// Total requests processed
    total_requests: Arc<AtomicU64>,
    /// Total errors
    total_errors: Arc<AtomicU64>,
}

impl ApiGateway {
    /// Create a new API gateway
    pub fn new(requests_per_sec: u64) -> Self {
        Self {
            rate_limiter: RateLimiter::new(requests_per_sec, requests_per_sec as f64),
            start_time: SystemTime::now(),
            total_requests: Arc::new(AtomicU64::new(0)),
            total_errors: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Process an incoming request; returns true if allowed by rate limiter
    pub fn admit_request(&self) -> bool {
        self.total_requests.fetch_add(1, Ordering::SeqCst);
        self.rate_limiter.allow_request()
    }

    /// Record an error
    pub fn record_error(&self) {
        self.total_errors.fetch_add(1, Ordering::SeqCst);
    }

    /// Get health check status
    pub fn health_check(&self) -> HealthCheck {
        let uptime = self.start_time.elapsed().unwrap_or_default().as_secs();
        let total_req = self.total_requests.load(Ordering::SeqCst);
        let total_err = self.total_errors.load(Ordering::SeqCst);
        let error_rate = if total_req > 0 {
            (total_err as f64 / total_req as f64) * 100.0
        } else {
            0.0
        };

        let status = if error_rate > 10.0 {
            HealthStatus::Unhealthy
        } else if error_rate > 5.0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        let mut health = HealthCheck::new(status, uptime);
        health.test_requests = total_req;
        health.error_rate_percent = error_rate;
        health.message = format!(
            "Processed {} requests, {} errors ({:.1}%)",
            total_req, total_err, error_rate
        );

        info!("Health check: {}", health.message);
        health
    }

    /// Get rate limiter stats
    pub fn available_capacity(&self) -> u64 {
        self.rate_limiter.available_tokens()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(100, 10.0);
        assert_eq!(limiter.available_tokens(), 100);
    }

    #[test]
    fn test_rate_limiter_allows_burst() {
        let limiter = RateLimiter::new(5, 1.0);
        for i in 0..5 {
            assert!(limiter.allow_request(), "Request {} should be allowed", i);
        }
        assert!(!limiter.allow_request(), "6th request should be denied");
    }

    #[test]
    fn test_rate_limiter_denies_when_empty() {
        let limiter = RateLimiter::new(1, 0.0);
        assert!(limiter.allow_request());
        assert!(!limiter.allow_request());
    }

    #[test]
    fn test_health_status_display() {
        assert_eq!(format!("{}", HealthStatus::Healthy), "healthy");
        assert_eq!(format!("{}", HealthStatus::Degraded), "degraded");
        assert_eq!(format!("{}", HealthStatus::Unhealthy), "unhealthy");
    }

    #[test]
    fn test_health_check_http_status() {
        let health = HealthCheck::new(HealthStatus::Healthy, 100);
        assert_eq!(health.http_status_code(), 200);

        let degraded = HealthCheck::new(HealthStatus::Degraded, 100);
        assert_eq!(degraded.http_status_code(), 200);

        let unhealthy = HealthCheck::new(HealthStatus::Unhealthy, 10);
        assert_eq!(unhealthy.http_status_code(), 503);
    }

    #[test]
    fn test_api_gateway_creation() {
        let gateway = ApiGateway::new(100);
        assert!(gateway.admit_request());
    }

    #[test]
    fn test_api_gateway_rate_limiting() {
        let gateway = ApiGateway::new(2);
        assert!(gateway.admit_request());
        assert!(gateway.admit_request());
        assert!(!gateway.admit_request());
    }

    #[test]
    fn test_api_gateway_error_tracking() {
        let gateway = ApiGateway::new(100);
        gateway.admit_request();
        gateway.record_error();
        let health = gateway.health_check();
        assert!(health.error_rate_percent > 0.0);
    }

    #[test]
    fn test_health_check_readiness() {
        let health = HealthCheck::new(HealthStatus::Healthy, 10);
        assert!(health.is_ready());
        assert!(health.is_alive());
    }

    #[test]
    fn test_health_check_liveness() {
        let health = HealthCheck::new(HealthStatus::Unhealthy, 10);
        assert!(!health.is_ready());
        assert!(!health.is_alive());
    }
}
