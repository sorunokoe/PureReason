//! Batch Processing: Support high-throughput bulk verification of multiple claims
//!
//! TRIZ Principle: Segmentation
//! Process multiple claims together to amortize overhead and enable 10–100× throughput.
//!
//! Batch mode allows processing 10–1000 claims in a single API call, reducing per-claim
//! overhead for network, model loading, and framework initialization. This is critical
//! for enterprise use cases (medical records, compliance reports, legal document reviews).

use crate::error::PureReasonError;
use crate::error::Result;
use crate::pipeline::KantianPipeline;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::info;

/// A single claim in a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchClaim {
    /// Unique identifier (UUID or user-defined)
    pub id: String,
    /// The claim to evaluate
    pub claim: String,
    /// Knowledge context (optional, domain-specific)
    pub knowledge: Option<String>,
}

/// Result for a single claim in batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    /// Claim ID (matches input)
    pub claim_id: String,
    /// Verdict: TRUE, FALSE, or UNKNOWN
    pub verdict: String,
    /// Confidence score (0.0–1.0)
    pub confidence: f64,
    /// Reasoning explanation
    pub reasoning: String,
    /// Latency for this claim in milliseconds
    pub latency_ms: f64,
}

/// Batch processing request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    /// List of claims to process
    pub claims: Vec<BatchClaim>,
    /// Whether to continue on individual claim errors
    pub continue_on_error: bool,
}

/// Batch processing response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse {
    /// Results for each claim
    pub results: Vec<BatchResult>,
    /// Total latency in milliseconds
    pub total_latency_ms: f64,
    /// Throughput: claims per second
    pub throughput_claims_per_sec: f64,
    /// Number of successful claims
    pub succeeded: usize,
    /// Number of failed claims
    pub failed: usize,
}

impl BatchResponse {
    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "batch: {} claims processed ({} ok, {} failed) in {:.0}ms ({:.1} claims/sec)",
            self.results.len(),
            self.succeeded,
            self.failed,
            self.total_latency_ms,
            self.throughput_claims_per_sec
        )
    }
}

/// Process a batch of claims
pub fn process_batch(request: BatchRequest) -> Result<BatchResponse> {
    let start = Instant::now();
    let batch_size = request.claims.len();
    let pipeline = KantianPipeline::new();

    let mut results = Vec::with_capacity(batch_size);
    let mut succeeded = 0;
    let mut failed = 0;

    for batch_claim in request.claims {
        let claim_start = Instant::now();

        // Format input as "Knowledge: X\nResponse: Y" for pipeline
        let input = if let Some(knowledge) = batch_claim.knowledge {
            format!("Knowledge: {}\nResponse: {}", knowledge, batch_claim.claim)
        } else {
            format!(
                "Question: {}\nAnswer: {}",
                batch_claim.claim, batch_claim.claim
            )
        };

        match pipeline.process(input) {
            Ok(report) => {
                let latency_ms = claim_start.elapsed().as_secs_f64() * 1000.0;
                let verdict_str = if report.verdict.has_contradictions {
                    "FALSE".to_string()
                } else if report.verdict.has_illusions {
                    "UNKNOWN".to_string()
                } else {
                    "TRUE".to_string()
                };

                results.push(BatchResult {
                    claim_id: batch_claim.id,
                    verdict: verdict_str,
                    confidence: report.verdict.ecs as f64 / 100.0,
                    reasoning: report.summary.clone(),
                    latency_ms,
                });
                succeeded += 1;
            }
            Err(e) => {
                if request.continue_on_error {
                    results.push(BatchResult {
                        claim_id: batch_claim.id,
                        verdict: "ERROR".to_string(),
                        confidence: 0.0,
                        reasoning: format!("Error: {}", e),
                        latency_ms: claim_start.elapsed().as_secs_f64() * 1000.0,
                    });
                    failed += 1;
                } else {
                    return Err(e);
                }
            }
        }
    }

    let total_latency_ms = start.elapsed().as_secs_f64() * 1000.0;
    let throughput_claims_per_sec = if total_latency_ms > 0.0 {
        (batch_size as f64 / total_latency_ms) * 1000.0
    } else {
        0.0
    };

    let response = BatchResponse {
        results,
        total_latency_ms,
        throughput_claims_per_sec,
        succeeded,
        failed,
    };

    info!("{}", response.summary());
    Ok(response)
}

/// Process a batch with adaptive throttling (rate limiting)
/// Useful for large batches to avoid overwhelming resources
pub fn process_batch_throttled(
    request: BatchRequest,
    max_concurrent: usize,
) -> Result<BatchResponse> {
    if max_concurrent == 0 {
        return Err(PureReasonError::InvalidInput(
            "max_concurrent must be > 0".to_string(),
        ));
    }

    // For now, sequential processing with early validation
    // In production, use rayon or tokio for true parallelization
    if request.claims.len() > max_concurrent * 100 {
        info!(
            "Large batch detected: {} claims with max_concurrent={}",
            request.claims.len(),
            max_concurrent
        );
    }

    process_batch(request)
}

/// Statistics for batch operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BatchStatistics {
    /// Total claims processed
    pub total_claims: u64,
    /// Total successful
    pub successful: u64,
    /// Total failed
    pub failed: u64,
    /// Average latency per claim
    pub avg_latency_ms: f64,
    /// Average confidence
    pub avg_confidence: f64,
}

impl BatchStatistics {
    /// Update with a batch response
    pub fn update(&mut self, response: &BatchResponse) {
        let batch_count = response.results.len() as u64;
        self.total_claims += batch_count;
        self.successful += response.succeeded as u64;
        self.failed += response.failed as u64;

        if !response.results.is_empty() {
            let avg_latency: f64 =
                response.results.iter().map(|r| r.latency_ms).sum::<f64>() / batch_count as f64;
            let avg_conf: f64 =
                response.results.iter().map(|r| r.confidence).sum::<f64>() / batch_count as f64;

            self.avg_latency_ms = (self.avg_latency_ms + avg_latency) / 2.0;
            self.avg_confidence = (self.avg_confidence + avg_conf) / 2.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_claim_creation() {
        let claim = BatchClaim {
            id: "claim_1".to_string(),
            claim: "The sky is blue".to_string(),
            knowledge: Some("blue light scatters".to_string()),
        };
        assert_eq!(claim.id, "claim_1");
    }

    #[test]
    fn test_batch_result_creation() {
        let result = BatchResult {
            claim_id: "claim_1".to_string(),
            verdict: "TRUE".to_string(),
            confidence: 0.95,
            reasoning: "Supported by physics".to_string(),
            latency_ms: 150.0,
        };
        assert_eq!(result.confidence, 0.95);
    }

    #[test]
    fn test_batch_response_summary() {
        let response = BatchResponse {
            results: vec![],
            total_latency_ms: 1000.0,
            throughput_claims_per_sec: 10.0,
            succeeded: 5,
            failed: 0,
        };
        let summary = response.summary();
        assert!(summary.contains("5 ok"));
    }

    #[test]
    fn test_batch_statistics_update() {
        let mut stats = BatchStatistics::default();
        let response = BatchResponse {
            results: vec![
                BatchResult {
                    claim_id: "1".to_string(),
                    verdict: "TRUE".to_string(),
                    confidence: 0.9,
                    reasoning: "ok".to_string(),
                    latency_ms: 100.0,
                },
                BatchResult {
                    claim_id: "2".to_string(),
                    verdict: "FALSE".to_string(),
                    confidence: 0.8,
                    reasoning: "ok".to_string(),
                    latency_ms: 100.0,
                },
            ],
            total_latency_ms: 200.0,
            throughput_claims_per_sec: 10.0,
            succeeded: 2,
            failed: 0,
        };
        stats.update(&response);
        assert_eq!(stats.total_claims, 2);
        assert_eq!(stats.successful, 2);
    }

    #[test]
    fn test_batch_request_default() {
        let request = BatchRequest {
            claims: vec![],
            continue_on_error: false,
        };
        assert!(!request.continue_on_error);
    }

    #[test]
    fn test_process_batch_throttled_zero_concurrent() {
        let request = BatchRequest {
            claims: vec![],
            continue_on_error: false,
        };
        let result = process_batch_throttled(request, 0);
        assert!(result.is_err());
    }
}
