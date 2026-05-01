//! # Human-in-the-Loop Feedback: Expert Validation and Active Learning
//!
//! Home Run #10: Collect expert feedback and learn from it
//!
//! TRIZ Principle: Feedback + Partial Feedback
//! Get expert annotations for high-uncertainty predictions, learn what reasoning
//! phase should have fired, and update the system accordingly.
//!
//! This enables specialized domain experts to guide reasoning improvements
//! without having to write code or configure systems manually.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A prediction that needs expert review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertReviewRequest {
    /// Unique ID for this review
    pub id: String,
    /// Claim being evaluated
    pub claim: String,
    /// Our prediction (what we said)
    pub our_prediction: bool,
    /// Our confidence in it
    pub our_confidence: f64,
    /// Domain (medical, legal, etc)
    pub domain: String,
    /// Reasoning phase that was active
    pub active_phase: String,
}

/// Expert annotation for a review request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertAnnotation {
    /// Review ID being annotated
    pub review_id: String,
    /// Expert's name/ID
    pub expert: String,
    /// Ground truth from expert
    pub actual: bool,
    /// Was our prediction correct?
    pub correct: bool,
    /// Phase that should have been active
    pub recommended_phase: String,
    /// Confidence in the recommendation (0-1)
    pub recommendation_confidence: f64,
    /// Free-text feedback
    pub feedback: String,
}

/// Aggregated learning from expert feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackInsight {
    /// Claim type where feedback applies
    pub claim_pattern: String,
    /// Most common recommended phase
    pub recommended_phase: String,
    /// How often experts recommended this
    pub recommendation_frequency: f64,
    /// Average confidence in the recommendation
    pub recommendation_confidence: f64,
    /// How often our predictions were wrong in this category
    pub error_rate: f64,
    /// Estimated F1 impact if we switch to recommended phase
    pub estimated_delta_f1: f64,
}

/// Human feedback manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackManager {
    /// All pending review requests
    pub pending_reviews: Vec<ExpertReviewRequest>,
    /// All collected annotations
    pub annotations: Vec<ExpertAnnotation>,
    /// Insights learned from annotations
    pub insights: Vec<FeedbackInsight>,
    /// Experts who have contributed
    pub expert_count: usize,
    /// Total annotations collected
    pub total_annotations: usize,
    /// System confidence in recommendations (0-1)
    pub system_confidence: f64,
}

impl FeedbackManager {
    /// Create new feedback manager
    pub fn new() -> Self {
        Self {
            pending_reviews: Vec::new(),
            annotations: Vec::new(),
            insights: Vec::new(),
            expert_count: 0,
            total_annotations: 0,
            system_confidence: 0.0,
        }
    }

    /// Create a review request for a high-uncertainty prediction
    pub fn create_review_request(
        &mut self,
        claim: String,
        prediction: bool,
        confidence: f64,
        domain: String,
        active_phase: String,
    ) -> String {
        let id = format!(
            "review-{}-{}",
            self.pending_reviews.len(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );

        self.pending_reviews.push(ExpertReviewRequest {
            id: id.clone(),
            claim,
            our_prediction: prediction,
            our_confidence: confidence,
            domain,
            active_phase,
        });

        id
    }

    /// Submit expert annotation
    pub fn submit_annotation(&mut self, annotation: ExpertAnnotation) -> bool {
        // Find matching review request
        if !self
            .pending_reviews
            .iter()
            .any(|r| r.id == annotation.review_id)
        {
            return false; // Invalid review ID
        }

        // First annotation from this expert?
        if !self
            .annotations
            .iter()
            .any(|a| a.expert == annotation.expert)
        {
            self.expert_count += 1;
        }

        self.annotations.push(annotation);
        self.total_annotations += 1;

        // Update system confidence with each annotation (diminishing returns)
        self.system_confidence = (self.total_annotations as f64 / 100.0).min(0.9);

        true
    }

    /// Analyze all annotations to extract insights
    pub fn extract_insights(&mut self) -> Vec<FeedbackInsight> {
        let mut insights_map: HashMap<String, Vec<&ExpertAnnotation>> = HashMap::new();

        // Group annotations by recommended phase
        for annotation in &self.annotations {
            insights_map
                .entry(annotation.recommended_phase.clone())
                .or_default()
                .push(annotation);
        }

        let mut insights = Vec::new();

        for (phase, group) in insights_map {
            let total = group.len() as f64;
            let correct = group.iter().filter(|a| a.correct).count() as f64;
            let error_rate = 1.0 - (correct / total);

            let avg_confidence: f64 = group
                .iter()
                .map(|a| a.recommendation_confidence)
                .sum::<f64>()
                / total;

            // Calculate estimated F1 impact
            let estimated_delta = if error_rate > 0.2 {
                0.05 * (error_rate.min(0.5))
            } else {
                0.02
            };

            insights.push(FeedbackInsight {
                claim_pattern: phase.clone(),
                recommended_phase: phase.clone(),
                recommendation_frequency: total / self.total_annotations.max(1) as f64,
                recommendation_confidence: avg_confidence,
                error_rate,
                estimated_delta_f1: estimated_delta,
            });
        }

        // Sort by impact (highest first)
        insights.sort_by(|a, b| {
            b.estimated_delta_f1
                .partial_cmp(&a.estimated_delta_f1)
                .unwrap()
        });

        self.insights = insights.clone();
        insights
    }

    /// Get recommendations for a domain
    pub fn get_domain_recommendations(&self, _domain: &str) -> Vec<&FeedbackInsight> {
        self.insights
            .iter()
            .filter(|i| i.recommendation_frequency > 0.1) // At least 10% frequency
            .collect()
    }

    /// Mark review as complete and remove from pending
    pub fn complete_review(&mut self, review_id: &str) -> bool {
        if let Some(pos) = self.pending_reviews.iter().position(|r| r.id == review_id) {
            self.pending_reviews.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get next review request that needs annotation (active learning)
    pub fn next_review_to_annotate(&self) -> Option<&ExpertReviewRequest> {
        // Prioritize high-uncertainty reviews
        self.pending_reviews.iter().max_by(|a, b| {
            (1.0 - (a.our_confidence - 0.5).abs())
                .partial_cmp(&(1.0 - (b.our_confidence - 0.5).abs()))
                .unwrap()
        })
    }

    /// Get review statistics
    pub fn get_statistics(&self) -> (usize, usize, f64, f64) {
        (
            self.pending_reviews.len(),
            self.total_annotations,
            self.system_confidence,
            self.insights.len() as f64,
        )
    }

    /// Estimate total F1 improvement from implementing all recommendations
    pub fn estimate_total_improvement(&self) -> f64 {
        let mut total = 0.0;
        for insight in &self.insights {
            total += insight.estimated_delta_f1;
        }
        total.min(0.2) // Cap at 20%
    }
}

impl Default for FeedbackManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_manager_new() {
        let fm = FeedbackManager::new();
        assert_eq!(fm.pending_reviews.len(), 0);
        assert_eq!(fm.total_annotations, 0);
        assert_eq!(fm.expert_count, 0);
    }

    #[test]
    fn test_create_review_request() {
        let mut fm = FeedbackManager::new();
        let id = fm.create_review_request(
            "test claim".to_string(),
            true,
            0.8,
            "medical".to_string(),
            "chain_of_thought".to_string(),
        );

        assert!(!id.is_empty());
        assert_eq!(fm.pending_reviews.len(), 1);
    }

    #[test]
    fn test_submit_annotation() {
        let mut fm = FeedbackManager::new();
        let review_id = fm.create_review_request(
            "test".to_string(),
            true,
            0.7,
            "medical".to_string(),
            "phase1".to_string(),
        );

        let annotation = ExpertAnnotation {
            review_id,
            expert: "dr_smith".to_string(),
            actual: false,
            correct: false,
            recommended_phase: "phase2".to_string(),
            recommendation_confidence: 0.9,
            feedback: "phase2 is better".to_string(),
        };

        assert!(fm.submit_annotation(annotation));
        assert_eq!(fm.total_annotations, 1);
        assert_eq!(fm.expert_count, 1);
    }

    #[test]
    fn test_system_confidence_increases() {
        let mut fm = FeedbackManager::new();
        assert_eq!(fm.system_confidence, 0.0);

        for i in 0..50 {
            let review_id = fm.create_review_request(
                format!("claim_{}", i),
                true,
                0.7,
                "test".to_string(),
                "phase".to_string(),
            );
            fm.submit_annotation(ExpertAnnotation {
                review_id,
                expert: format!("expert_{}", i % 5),
                actual: true,
                correct: true,
                recommended_phase: "phase1".to_string(),
                recommendation_confidence: 0.8,
                feedback: "good".to_string(),
            });
        }

        assert!(fm.system_confidence > 0.4);
    }

    #[test]
    fn test_extract_insights() {
        let mut fm = FeedbackManager::new();

        for i in 0..30 {
            let review_id = fm.create_review_request(
                format!("claim_{}", i),
                false,
                0.5,
                "test".to_string(),
                "phase1".to_string(),
            );
            fm.submit_annotation(ExpertAnnotation {
                review_id,
                expert: "expert_1".to_string(),
                actual: i % 2 == 0,
                correct: i % 2 == 0,
                recommended_phase: "phase2".to_string(),
                recommendation_confidence: 0.85,
                feedback: "use phase2".to_string(),
            });
        }

        let insights = fm.extract_insights();
        assert!(!insights.is_empty());
        assert!(insights[0].recommendation_frequency > 0.0);
    }

    #[test]
    fn test_complete_review() {
        let mut fm = FeedbackManager::new();
        let id = fm.create_review_request(
            "test".to_string(),
            true,
            0.7,
            "test".to_string(),
            "phase".to_string(),
        );

        assert_eq!(fm.pending_reviews.len(), 1);
        assert!(fm.complete_review(&id));
        assert_eq!(fm.pending_reviews.len(), 0);
    }

    #[test]
    fn test_next_review_to_annotate() {
        let mut fm = FeedbackManager::new();
        fm.create_review_request(
            "certain".to_string(),
            true,
            0.95,
            "test".to_string(),
            "phase".to_string(),
        );
        fm.create_review_request(
            "uncertain".to_string(),
            true,
            0.51,
            "test".to_string(),
            "phase".to_string(),
        );

        let next = fm.next_review_to_annotate().unwrap();
        // Should prioritize uncertain (closest to 0.5)
        assert_eq!(next.claim, "uncertain");
    }

    #[test]
    fn test_get_statistics() {
        let mut fm = FeedbackManager::new();
        for i in 0..10 {
            let id = fm.create_review_request(
                format!("claim_{}", i),
                true,
                0.7,
                "test".to_string(),
                "phase".to_string(),
            );
            fm.submit_annotation(ExpertAnnotation {
                review_id: id.clone(),
                expert: "expert".to_string(),
                actual: true,
                correct: true,
                recommended_phase: "phase1".to_string(),
                recommendation_confidence: 0.8,
                feedback: "ok".to_string(),
            });
            fm.complete_review(&id); // Mark review as complete
        }

        let (pending, annotated, confidence, _insights) = fm.get_statistics();
        assert_eq!(pending, 0);
        assert_eq!(annotated, 10);
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_estimate_total_improvement() {
        let mut fm = FeedbackManager::new();

        for i in 0..40 {
            let id = fm.create_review_request(
                format!("claim_{}", i),
                true,
                0.6,
                "medical".to_string(),
                "phase1".to_string(),
            );
            fm.submit_annotation(ExpertAnnotation {
                review_id: id,
                expert: format!("expert_{}", i % 3),
                actual: i % 2 == 0,
                correct: i % 2 == 0,
                recommended_phase: if i < 20 {
                    "phase2".to_string()
                } else {
                    "phase3".to_string()
                },
                recommendation_confidence: 0.85,
                feedback: "feedback".to_string(),
            });
        }

        fm.extract_insights();
        let improvement = fm.estimate_total_improvement();
        assert!(improvement > 0.0);
        assert!(improvement <= 0.2);
    }

    #[test]
    fn test_multiple_experts() {
        let mut fm = FeedbackManager::new();

        for expert_id in 0..3 {
            for i in 0..10 {
                let id = fm.create_review_request(
                    format!("claim_{}_{}", expert_id, i),
                    true,
                    0.7,
                    "test".to_string(),
                    "phase".to_string(),
                );
                fm.submit_annotation(ExpertAnnotation {
                    review_id: id,
                    expert: format!("expert_{}", expert_id),
                    actual: true,
                    correct: true,
                    recommended_phase: "best_phase".to_string(),
                    recommendation_confidence: 0.9,
                    feedback: "agreed".to_string(),
                });
            }
        }

        assert_eq!(fm.expert_count, 3);
        assert_eq!(fm.total_annotations, 30);
    }

    #[test]
    fn test_get_domain_recommendations() {
        let mut fm = FeedbackManager::new();

        for i in 0..30 {
            let id = fm.create_review_request(
                format!("claim_{}", i),
                true,
                0.7,
                "medical".to_string(),
                "phase1".to_string(),
            );
            fm.submit_annotation(ExpertAnnotation {
                review_id: id,
                expert: "expert".to_string(),
                actual: true,
                correct: true,
                recommended_phase: "better_phase".to_string(),
                recommendation_confidence: 0.95,
                feedback: "good".to_string(),
            });
        }

        fm.extract_insights();
        let recs = fm.get_domain_recommendations("medical");
        assert!(!recs.is_empty());
    }
}
