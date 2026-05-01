//! # Competitive Analysis: Benchmark Domination Strategy Report Generator
//!
//! Analyzes PureReason performance vs competitors and generates
//! strategic insights for winning each benchmark category.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Competitive dimension (accuracy, speed, cost, explainability, etc)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitiveDimension {
    /// Dimension name
    pub name: String,
    /// Our score (0-100)
    pub our_score: f64,
    /// Competitor scores
    pub competitor_scores: HashMap<String, f64>,
    /// Weight in overall ranking (0-1)
    pub weight: f64,
    /// Advantage description
    pub advantage: String,
}

impl CompetitiveDimension {
    /// Compute weighted advantage
    pub fn weighted_advantage(&self) -> f64 {
        let avg_competitor: f64 = if self.competitor_scores.is_empty() {
            0.0
        } else {
            self.competitor_scores.values().sum::<f64>() / self.competitor_scores.len() as f64
        };

        let raw_advantage = self.our_score - avg_competitor;
        raw_advantage * self.weight
    }
}

/// Strategy for winning a specific benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinningStrategy {
    /// Benchmark name
    pub benchmark: String,
    /// Key to winning
    pub winning_factor: String,
    /// Which strategic wins help most
    pub key_wins: Vec<String>,
    /// Estimated F1 improvement
    pub estimated_f1_gain: f64,
    /// Confidence in strategy (0-1)
    pub confidence: f64,
}

/// Competitive positioning report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitiveReport {
    /// Benchmarks analyzed
    pub benchmarks: Vec<String>,
    /// Our average F1
    pub our_f1: f64,
    /// Competitor F1s
    pub competitor_f1s: HashMap<String, f64>,
    /// Competitive dimensions
    pub dimensions: Vec<CompetitiveDimension>,
    /// Winning strategies per benchmark
    pub strategies: Vec<WinningStrategy>,
    /// Overall ranking (1=best)
    pub our_ranking: usize,
    /// Total competitors analyzed
    pub competitor_count: usize,
    /// Market positioning
    pub positioning: String,
}

impl CompetitiveReport {
    /// Create new report
    pub fn new(our_f1: f64) -> Self {
        Self {
            benchmarks: Vec::new(),
            our_f1,
            competitor_f1s: HashMap::new(),
            dimensions: Vec::new(),
            strategies: Vec::new(),
            our_ranking: 0,
            competitor_count: 0,
            positioning: String::new(),
        }
    }

    /// Add benchmark
    pub fn add_benchmark(&mut self, name: String) {
        self.benchmarks.push(name);
    }

    /// Add competitor F1
    pub fn add_competitor_f1(&mut self, name: String, f1: f64) {
        self.competitor_f1s.insert(name, f1);
        self.competitor_count = self.competitor_f1s.len();
    }

    /// Add competitive dimension
    pub fn add_dimension(&mut self, dimension: CompetitiveDimension) {
        self.dimensions.push(dimension);
    }

    /// Add winning strategy
    pub fn add_strategy(&mut self, strategy: WinningStrategy) {
        self.strategies.push(strategy);
    }

    /// Compute overall ranking
    pub fn compute_ranking(&mut self) {
        let mut better_count = 0;

        for (_name, f1) in &self.competitor_f1s {
            if self.our_f1 > *f1 {
                better_count += 1;
            }
        }

        self.our_ranking = (self.competitor_count - better_count) + 1;
    }

    /// Set market positioning
    pub fn set_positioning(&mut self, positioning: String) {
        self.positioning = positioning;
    }

    /// Generate executive summary
    pub fn executive_summary(&self) -> String {
        format!(
            "PureReason F1: {:.2} | Ranking: #{}/{} competitors | Key advantages: determinism, explainability, cost | {} | Recommended focus: {}",
            self.our_f1,
            self.our_ranking,
            self.competitor_count.max(1),
            self.positioning,
            if self.strategies.is_empty() {
                "multi-domain excellence".to_string()
            } else {
                self.strategies[0].winning_factor.clone()
            }
        )
    }

    /// Get top N winning strategies
    pub fn top_strategies(&self, n: usize) -> Vec<&WinningStrategy> {
        let mut strategies = self.strategies.iter().collect::<Vec<_>>();
        strategies.sort_by(|a, b| {
            b.estimated_f1_gain
                .partial_cmp(&a.estimated_f1_gain)
                .unwrap()
        });
        strategies.into_iter().take(n).collect()
    }

    /// Get weighted competitive advantage
    pub fn weighted_advantage(&self) -> f64 {
        self.dimensions.iter().map(|d| d.weighted_advantage()).sum()
    }
}

/// Market segment analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSegment {
    /// Segment name (regulated, academic, enterprise, etc)
    pub name: String,
    /// PureReason fit score (0-100)
    pub fit_score: f64,
    /// Key competitors in this segment
    pub competitors: Vec<String>,
    /// Our competitive advantage here
    pub advantage: String,
    /// Market size (est. revenue potential)
    pub market_size: String,
}

/// Benchmark domination analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DominationAnalysis {
    /// Date of analysis
    pub date: String,
    /// Competitive report
    pub report: CompetitiveReport,
    /// Market segments
    pub segments: Vec<MarketSegment>,
    /// Recommended focus areas
    pub focus_areas: Vec<String>,
    /// 90-day domination plan
    pub domination_plan: Vec<String>,
}

impl DominationAnalysis {
    /// Create new analysis
    pub fn new(our_f1: f64) -> Self {
        Self {
            date: format_date(),
            report: CompetitiveReport::new(our_f1),
            segments: Vec::new(),
            focus_areas: Vec::new(),
            domination_plan: Vec::new(),
        }
    }

    /// Add market segment
    pub fn add_segment(&mut self, segment: MarketSegment) {
        self.segments.push(segment);
    }

    /// Add focus area
    pub fn add_focus(&mut self, focus: String) {
        self.focus_areas.push(focus);
    }

    /// Add domination step
    pub fn add_domination_step(&mut self, step: String) {
        self.domination_plan.push(step);
    }

    /// Generate comprehensive report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str(&format!(
            "# PureReason Competitive Analysis Report\n\n**Date**: {}\n\n",
            self.date
        ));

        report.push_str("## Executive Summary\n");
        report.push_str(&format!("{}\n\n", self.report.executive_summary()));

        report.push_str("## Market Segments\n");
        for segment in &self.segments {
            report.push_str(&format!(
                "- **{}**: Fit={}/100, Advantage: {}\n",
                segment.name, segment.fit_score, segment.advantage
            ));
        }

        report.push_str("\n## 90-Day Domination Plan\n");
        for (i, step) in self.domination_plan.iter().enumerate() {
            report.push_str(&format!("{}. {}\n", i + 1, step));
        }

        report
    }
}

/// Helper to format date
fn format_date() -> String {
    "2026-04-30".to_string() // Static date for reproducibility
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_competitive_dimension_new() {
        let dim = CompetitiveDimension {
            name: "Accuracy".to_string(),
            our_score: 85.0,
            competitor_scores: {
                let mut m = HashMap::new();
                m.insert("o3".to_string(), 82.0);
                m
            },
            weight: 0.4,
            advantage: "2-3% higher accuracy".to_string(),
        };

        assert_eq!(dim.our_score, 85.0);
        assert!(dim.weighted_advantage() > 0.0);
    }

    #[test]
    fn test_winning_strategy() {
        let strategy = WinningStrategy {
            benchmark: "gsm8k".to_string(),
            winning_factor: "Mathematical solver".to_string(),
            key_wins: vec!["math_solver".to_string(), "multi_hop_reasoner".to_string()],
            estimated_f1_gain: 0.08,
            confidence: 0.85,
        };

        assert_eq!(strategy.benchmark, "gsm8k");
        assert_eq!(strategy.estimated_f1_gain, 0.08);
    }

    #[test]
    fn test_competitive_report_new() {
        let report = CompetitiveReport::new(0.87);
        assert_eq!(report.our_f1, 0.87);
        assert_eq!(report.competitor_count, 0);
    }

    #[test]
    fn test_add_competitor() {
        let mut report = CompetitiveReport::new(0.87);
        report.add_competitor_f1("o3".to_string(), 0.90);
        report.add_competitor_f1("R1".to_string(), 0.80);

        assert_eq!(report.competitor_count, 2);
    }

    #[test]
    fn test_compute_ranking() {
        let mut report = CompetitiveReport::new(0.87);
        report.add_competitor_f1("o3".to_string(), 0.85);
        report.add_competitor_f1("R1".to_string(), 0.88);
        report.add_competitor_f1("EVICheck".to_string(), 0.82);

        report.compute_ranking();
        assert!(report.our_ranking > 0 && report.our_ranking <= 3);
    }

    #[test]
    fn test_executive_summary() {
        let report = CompetitiveReport::new(0.87);
        let summary = report.executive_summary();
        assert!(summary.contains("0.87"));
        assert!(summary.contains("determinism"));
    }

    #[test]
    fn test_domination_analysis() {
        let mut analysis = DominationAnalysis::new(0.87);
        analysis.add_focus("Math reasoning".to_string());
        analysis.add_domination_step("Implement multi-hop chains".to_string());

        assert_eq!(analysis.focus_areas.len(), 1);
        assert_eq!(analysis.domination_plan.len(), 1);
    }

    #[test]
    fn test_market_segment() {
        let segment = MarketSegment {
            name: "Regulated enterprises".to_string(),
            fit_score: 95.0,
            competitors: vec!["o3".to_string()],
            advantage: "100% determinism + explainability".to_string(),
            market_size: "$10B+".to_string(),
        };

        assert_eq!(segment.fit_score, 95.0);
    }

    #[test]
    fn test_top_strategies() {
        let mut report = CompetitiveReport::new(0.87);
        report.add_strategy(WinningStrategy {
            benchmark: "gsm8k".to_string(),
            winning_factor: "Math".to_string(),
            key_wins: vec![],
            estimated_f1_gain: 0.05,
            confidence: 0.8,
        });
        report.add_strategy(WinningStrategy {
            benchmark: "mmlu".to_string(),
            winning_factor: "Knowledge".to_string(),
            key_wins: vec![],
            estimated_f1_gain: 0.08,
            confidence: 0.85,
        });

        let top = report.top_strategies(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].estimated_f1_gain, 0.08);
    }

    #[test]
    fn test_generate_report() {
        let analysis = DominationAnalysis::new(0.87);
        let report_text = analysis.generate_report();
        assert!(report_text.contains("Competitive Analysis Report"));
    }
}
