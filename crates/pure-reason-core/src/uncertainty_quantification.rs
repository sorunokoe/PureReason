//! Uncertainty Quantification: Represent confidence as ranges, not point estimates
//!
//! TRIZ Principle: Transition to Micro-Level + Measurement
//! Replace single scores with Bayesian confidence intervals, propagate uncertainty
//! through reasoning chains, and tag uncertainty sources.
//!
//! Inspired by forecasting methodology and Bayesian reasoning, but deterministic.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Confidence interval with lower, point, and upper estimates
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    /// Lower bound (e.g., 5th percentile)
    pub lower: f64,
    /// Point estimate (median or mode)
    pub point: f64,
    /// Upper bound (e.g., 95th percentile)
    pub upper: f64,
}

impl ConfidenceInterval {
    /// Create a confidence interval
    pub fn new(lower: f64, point: f64, upper: f64) -> Self {
        debug_assert!(
            lower <= point && point <= upper,
            "Invalid confidence interval"
        );
        debug_assert!(
            lower >= 0.0 && upper <= 1.0,
            "Confidence must be between 0 and 1"
        );
        Self {
            lower,
            point,
            upper,
        }
    }

    /// Create a point estimate (zero uncertainty)
    pub fn point_estimate(confidence: f64) -> Self {
        Self {
            lower: confidence,
            point: confidence,
            upper: confidence,
        }
    }

    /// Width of the confidence interval (uncertainty measure)
    pub fn width(&self) -> f64 {
        self.upper - self.lower
    }

    /// Check if point is within interval
    pub fn contains(&self, value: f64) -> bool {
        value >= self.lower && value <= self.upper
    }

    /// Shift interval by a constant (e.g., applied bonus or penalty)
    pub fn shift(&self, delta: f64) -> Self {
        let lower = (self.lower + delta).clamp(0.0, 1.0);
        let point = (self.point + delta).clamp(0.0, 1.0);
        let upper = (self.upper + delta).clamp(0.0, 1.0);
        Self {
            lower,
            point,
            upper,
        }
    }

    /// Scale interval (e.g., when combining confidences)
    pub fn scale(&self, factor: f64) -> Self {
        Self {
            lower: (self.lower * factor).clamp(0.0, 1.0),
            point: (self.point * factor).clamp(0.0, 1.0),
            upper: (self.upper * factor).clamp(0.0, 1.0),
        }
    }

    /// Combine two intervals (AND logic: conservative)
    pub fn combine_and(&self, other: &ConfidenceInterval) -> Self {
        Self {
            lower: (self.lower * other.lower).clamp(0.0, 1.0),
            point: (self.point * other.point).clamp(0.0, 1.0),
            upper: (self.upper * other.upper).clamp(0.0, 1.0),
        }
    }

    /// Combine two intervals (OR logic: optimistic)
    pub fn combine_or(&self, other: &ConfidenceInterval) -> Self {
        Self {
            lower: (self.lower + other.lower - self.lower * other.lower).clamp(0.0, 1.0),
            point: (self.point + other.point - self.point * other.point).clamp(0.0, 1.0),
            upper: (self.upper + other.upper - self.upper * other.upper).clamp(0.0, 1.0),
        }
    }
}

impl fmt::Display for ConfidenceInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:.1}% [{:.1}%-{:.1}%]",
            self.point * 100.0,
            self.lower * 100.0,
            self.upper * 100.0
        )
    }
}

/// Source of uncertainty
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UncertaintySource {
    /// Empirical/measurement uncertainty (data variance)
    Empirical,
    /// Logical/deductive uncertainty (incomplete information)
    Logical,
    /// Domain expertise uncertainty (human disagreement)
    Domain,
    /// Model output uncertainty (LLM stochasticity)
    Model,
    /// Unknown source of uncertainty
    Unknown,
}

impl fmt::Display for UncertaintySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empirical => write!(f, "Empirical"),
            Self::Logical => write!(f, "Logical"),
            Self::Domain => write!(f, "Domain"),
            Self::Model => write!(f, "Model"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Confidence with uncertainty attribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributedConfidence {
    /// Confidence interval
    pub interval: ConfidenceInterval,
    /// Source of uncertainty
    pub source: UncertaintySource,
    /// Explanation for uncertainty
    pub reason: String,
}

impl AttributedConfidence {
    /// Create a new attributed confidence
    pub fn new(interval: ConfidenceInterval, source: UncertaintySource, reason: String) -> Self {
        Self {
            interval,
            source,
            reason,
        }
    }

    /// Summarize as string
    pub fn summary(&self) -> String {
        format!("{} ({}): {}", self.interval, self.source, self.reason)
    }
}

impl fmt::Display for AttributedConfidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
    }
}

/// Propagates uncertainty through a reasoning chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertaintyChain {
    /// Confidences at each step
    pub step_confidences: Vec<AttributedConfidence>,
    /// Aggregated confidence (product of point estimates)
    pub aggregated: ConfidenceInterval,
    /// Weakest link
    pub weakest_point: f64,
    /// Strongest link
    pub strongest_point: f64,
}

impl UncertaintyChain {
    /// Create from list of attributed confidences
    pub fn from_steps(steps: Vec<AttributedConfidence>) -> Self {
        if steps.is_empty() {
            return Self {
                step_confidences: steps,
                aggregated: ConfidenceInterval::point_estimate(1.0),
                weakest_point: 1.0,
                strongest_point: 1.0,
            };
        }

        // Calculate aggregated (product of point estimates)
        let mut lower = 1.0;
        let mut point = 1.0;
        let mut upper = 1.0;

        for step in &steps {
            lower *= step.interval.lower;
            point *= step.interval.point;
            upper *= step.interval.upper;
        }

        let aggregated = ConfidenceInterval {
            lower: lower.clamp(0.0, 1.0),
            point: point.clamp(0.0, 1.0),
            upper: upper.clamp(0.0, 1.0),
        };

        let weakest_point = steps.iter().map(|s| s.interval.point).fold(1.0, f64::min);
        let strongest_point = steps.iter().map(|s| s.interval.point).fold(0.0, f64::max);

        Self {
            step_confidences: steps,
            aggregated,
            weakest_point,
            strongest_point,
        }
    }

    /// Total uncertainty width in the chain
    pub fn total_uncertainty(&self) -> f64 {
        self.aggregated.width()
    }

    /// Is chain sufficiently certain?
    pub fn is_certain(&self, threshold: f64) -> bool {
        self.aggregated.point >= threshold
    }

    /// Explain which step(s) contribute most to uncertainty
    pub fn uncertainty_breakdown(&self) -> String {
        let mut explanation = format!("Chain aggregated confidence: {}\n", self.aggregated);
        explanation.push_str("Uncertainty contributions by step:\n");

        for (idx, step) in self.step_confidences.iter().enumerate() {
            let width_contribution = step.interval.width();
            explanation.push_str(&format!(
                "  Step {}: {} (width={:.1}%)\n",
                idx,
                step.summary(),
                width_contribution * 100.0
            ));
        }

        explanation.push_str(&format!(
            "\nWeakest link: {:.1}%, Strongest: {:.1}%\n",
            self.weakest_point * 100.0,
            self.strongest_point * 100.0
        ));

        explanation
    }
}

impl fmt::Display for UncertaintyChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Chain: {} steps, aggregated={}, weakest={:.1}%",
            self.step_confidences.len(),
            self.aggregated,
            self.weakest_point * 100.0
        )
    }
}

/// Builder for constructing uncertainty chains
pub struct UncertaintyBuilder {
    steps: Vec<AttributedConfidence>,
}

impl UncertaintyBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self { steps: vec![] }
    }

    /// Add a step with empirical uncertainty
    pub fn empirical(mut self, lower: f64, point: f64, upper: f64, reason: String) -> Self {
        self.steps.push(AttributedConfidence::new(
            ConfidenceInterval::new(lower, point, upper),
            UncertaintySource::Empirical,
            reason,
        ));
        self
    }

    /// Add a step with logical uncertainty
    pub fn logical(mut self, lower: f64, point: f64, upper: f64, reason: String) -> Self {
        self.steps.push(AttributedConfidence::new(
            ConfidenceInterval::new(lower, point, upper),
            UncertaintySource::Logical,
            reason,
        ));
        self
    }

    /// Add a step with domain expertise uncertainty
    pub fn domain(mut self, lower: f64, point: f64, upper: f64, reason: String) -> Self {
        self.steps.push(AttributedConfidence::new(
            ConfidenceInterval::new(lower, point, upper),
            UncertaintySource::Domain,
            reason,
        ));
        self
    }

    /// Add a step with model uncertainty
    pub fn model(mut self, lower: f64, point: f64, upper: f64, reason: String) -> Self {
        self.steps.push(AttributedConfidence::new(
            ConfidenceInterval::new(lower, point, upper),
            UncertaintySource::Model,
            reason,
        ));
        self
    }

    /// Build the chain
    pub fn build(self) -> UncertaintyChain {
        UncertaintyChain::from_steps(self.steps)
    }
}

impl Default for UncertaintyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_interval_creation() {
        let ci = ConfidenceInterval::new(0.70, 0.80, 0.90);
        assert_eq!(ci.lower, 0.70);
        assert_eq!(ci.point, 0.80);
        assert_eq!(ci.upper, 0.90);
    }

    #[test]
    fn test_confidence_interval_width() {
        let ci = ConfidenceInterval::new(0.70, 0.80, 0.90);
        assert!((ci.width() - 0.20).abs() < 1e-10);
    }

    #[test]
    fn test_point_estimate() {
        let ci = ConfidenceInterval::point_estimate(0.85);
        assert!((ci.width()).abs() < 1e-10);
    }

    #[test]
    fn test_confidence_contains() {
        let ci = ConfidenceInterval::new(0.70, 0.80, 0.90);
        assert!(ci.contains(0.80));
        assert!(!ci.contains(0.60));
    }

    #[test]
    fn test_confidence_shift() {
        let ci = ConfidenceInterval::new(0.70, 0.80, 0.90);
        let shifted = ci.shift(0.05);
        assert!((shifted.point - 0.85).abs() < 1e-10);
    }

    #[test]
    fn test_confidence_scale() {
        let ci = ConfidenceInterval::new(0.70, 0.80, 0.90);
        let scaled = ci.scale(0.5);
        assert!((scaled.point - 0.40).abs() < 1e-10);
    }

    #[test]
    fn test_combine_and() {
        let ci1 = ConfidenceInterval::new(0.70, 0.80, 0.90);
        let ci2 = ConfidenceInterval::new(0.60, 0.75, 0.85);
        let combined = ci1.combine_and(&ci2);
        assert!(combined.point < ci1.point && combined.point < ci2.point);
    }

    #[test]
    fn test_combine_or() {
        let ci1 = ConfidenceInterval::new(0.70, 0.80, 0.90);
        let ci2 = ConfidenceInterval::new(0.60, 0.75, 0.85);
        let combined = ci1.combine_or(&ci2);
        assert!(combined.point > ci1.point || combined.point > ci2.point);
    }

    #[test]
    fn test_attributed_confidence() {
        let ci = ConfidenceInterval::new(0.70, 0.80, 0.90);
        let ac = AttributedConfidence::new(
            ci,
            UncertaintySource::Empirical,
            "Data variance".to_string(),
        );
        assert_eq!(ac.source, UncertaintySource::Empirical);
    }

    #[test]
    fn test_uncertainty_chain_aggregation() {
        let chain = UncertaintyBuilder::new()
            .empirical(0.80, 0.90, 0.95, "Step 1".to_string())
            .empirical(0.75, 0.85, 0.92, "Step 2".to_string())
            .build();

        assert_eq!(chain.step_confidences.len(), 2);
        // 0.90 * 0.85 = 0.765, so aggregated should be less
        assert!(chain.aggregated.point < 0.77);
    }

    #[test]
    fn test_uncertainty_breakdown() {
        let chain = UncertaintyBuilder::new()
            .empirical(0.80, 0.90, 0.95, "Step 1".to_string())
            .logical(0.70, 0.80, 0.90, "Step 2".to_string())
            .build();

        let breakdown = chain.uncertainty_breakdown();
        assert!(breakdown.contains("Step 0"));
        assert!(breakdown.contains("Weakest link"));
    }

    #[test]
    fn test_uncertainty_source_display() {
        assert_eq!(format!("{}", UncertaintySource::Empirical), "Empirical");
        assert_eq!(format!("{}", UncertaintySource::Logical), "Logical");
    }

    #[test]
    fn test_builder_chaining() {
        let chain = UncertaintyBuilder::new()
            .empirical(0.80, 0.90, 0.95, "Empirical".to_string())
            .logical(0.70, 0.80, 0.90, "Logical".to_string())
            .domain(0.75, 0.85, 0.92, "Domain".to_string())
            .model(0.60, 0.75, 0.85, "Model".to_string())
            .build();

        assert_eq!(chain.step_confidences.len(), 4);
    }

    #[test]
    fn test_confidence_interval_display() {
        let ci = ConfidenceInterval::new(0.70, 0.80, 0.90);
        let s = format!("{}", ci);
        assert!(s.contains("80.0%"));
    }

    #[test]
    fn test_is_certain() {
        let chain = UncertaintyBuilder::new()
            .empirical(0.80, 0.90, 0.95, "Test".to_string())
            .build();

        assert!(chain.is_certain(0.80));
        assert!(!chain.is_certain(0.95));
    }
}
