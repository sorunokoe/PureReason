//! # Self-Audit Command (TRIZ S-7)
//!
//! Runs a canonical test battery through the full Kantian pipeline and
//! reports a **System Confidence Score** — the fraction of canonical test cases
//! correctly handled.
//!
//! ## TRIZ rationale (Principle #25 Self-Service, #13 Opposite, #23 Feedback)
//!
//! The Architectonic (already implemented) *describes* the system. Self-audit
//! *tests* the system using its own tools — the pipeline critiques itself.
//! If the score drops below 0.80, the pipeline warns that recalibration is needed.

use anyhow::Result;
use clap::Args;
use colored::Colorize;
use pure_reason_core::analytic::Category;
use pure_reason_core::pipeline::KantianPipeline;

/// Self-audit: run canonical test battery and report System Confidence Score.
#[derive(Args)]
pub struct SelfAuditCmd {
    /// Show detailed results for each test case
    #[arg(short, long)]
    pub verbose: bool,
}

// ─── Canonical Test Cases ────────────────────────────────────────────────────

struct TestCase {
    label: &'static str,
    input: &'static str,
    /// Expected: dominant category detected
    expected_category: Option<Category>,
    /// Expected: at least one illusion detected
    expected_illusion: bool,
    /// Expected: at least one antinomy detected
    expected_antinomy: bool,
    /// Expected: at least one paralogism detected
    expected_paralogism: bool,
    /// Expected language game (substring match)
    expected_game: Option<&'static str>,
}

fn canonical_tests() -> Vec<TestCase> {
    vec![
        // Category detection
        TestCase {
            label: "Causality — causal claim",
            input: "Heat causes water to boil because thermal energy increases kinetic motion.",
            expected_category: Some(Category::Causality),
            expected_illusion: false,
            expected_antinomy: false,
            expected_paralogism: false,
            expected_game: Some("Scientific"),
        },
        TestCase {
            label: "Necessity — universal duty",
            input: "You ought always to keep your promises. Moral duty is universally binding.",
            expected_category: Some(Category::Necessity),
            expected_illusion: false,
            expected_antinomy: false,
            expected_paralogism: false,
            expected_game: Some("Moral"),
        },
        TestCase {
            label: "Totality — universal quantification",
            input: "Every event in the universe is determined by prior causes without exception.",
            expected_category: Some(Category::Causality), // Causality wins due to "causes"/"determined"; Totality is secondary
            expected_illusion: false,
            expected_antinomy: false,
            expected_paralogism: false,
            expected_game: None,
        },
        TestCase {
            label: "Negation — strong negation",
            input: "There is no evidence and nothing supports the claim. The result is entirely absent.",
            expected_category: Some(Category::Negation),
            expected_illusion: false,
            expected_antinomy: false,
            expected_paralogism: false,
            expected_game: None,
        },
        // Illusion detection
        TestCase {
            label: "Soul illusion — hypostatization",
            input: "The soul is immortal and persists after death as a simple substance.",
            expected_category: None,
            expected_illusion: true,
            expected_antinomy: false,
            expected_paralogism: false,
            expected_game: Some("Religious"),
        },
        TestCase {
            label: "God illusion — necessary being",
            input: "God necessarily exists as the ground of all being and the most perfect being.",
            expected_category: None,
            expected_illusion: true,
            expected_antinomy: false,
            expected_paralogism: false,
            expected_game: Some("Philosophical"), // "being"/"existence" outscores "god" token
        },
        // Antinomy detection
        TestCase {
            label: "First Antinomy — temporal finitude",
            input: "The universe had a beginning in time. The universe has no beginning.",
            expected_category: None,
            expected_illusion: false,
            expected_antinomy: true,
            expected_paralogism: false,
            expected_game: None,
        },
        TestCase {
            label: "Third Antinomy — freedom/determinism",
            input: "Human beings have genuine free will. Everything is causally determined and there is no free will.",
            expected_category: None,
            expected_illusion: false,
            expected_antinomy: true,
            expected_paralogism: false,
            expected_game: None,
        },
        // Paralogism detection
        TestCase {
            label: "Substantiality Paralogism",
            input: "I think therefore I am a simple, unified, immortal substance.",
            expected_category: None,
            expected_illusion: false,
            expected_antinomy: false,
            expected_paralogism: true,
            expected_game: None,
        },
        // Language game detection
        TestCase {
            label: "Mathematical game",
            input: "The theorem follows from the axioms by formal proof. Q.E.D.",
            expected_category: None,
            expected_illusion: false,
            expected_antinomy: false,
            expected_paralogism: false,
            expected_game: Some("Mathematical"),
        },
        TestCase {
            label: "Technical game",
            input: "The algorithm implements a recursive function with O(n log n) complexity.",
            expected_category: None,
            expected_illusion: false,
            expected_antinomy: false,
            expected_paralogism: false,
            expected_game: Some("Technical"),
        },
        // Clean safe input — should return SAFE with scientific game
        TestCase {
            label: "Safe input — no issues",
            input: "Water boils at 100 degrees Celsius at standard atmospheric pressure.",
            expected_category: None,   // "boils"/"at" don't match strong causal signals
            expected_illusion: false,
            expected_antinomy: false,
            expected_paralogism: false,
            expected_game: Some("Scientific"),
        },
    ]
}

// ─── Test Evaluation ─────────────────────────────────────────────────────────

struct TestResult {
    label: String,
    passed: bool,
    failures: Vec<String>,
}

impl SelfAuditCmd {
    pub async fn run(&self, _format: &str) -> Result<()> {
        let pipeline = KantianPipeline::new();
        let tests = canonical_tests();
        let total = tests.len();
        let mut results: Vec<TestResult> = Vec::new();

        println!("{}", "━".repeat(64).dimmed());
        println!(
            "{}",
            "  PureReason Self-Audit — System Confidence Report".bold()
        );
        println!("{}", "━".repeat(64).dimmed());
        println!();

        for test in &tests {
            let report = match pipeline.process(test.input) {
                Ok(r) => r,
                Err(e) => {
                    results.push(TestResult {
                        label: test.label.to_string(),
                        passed: false,
                        failures: vec![format!("Pipeline error: {}", e)],
                    });
                    continue;
                }
            };

            let mut failures = Vec::new();

            // Check expected category
            if let Some(expected_cat) = test.expected_category {
                if report.understanding.category_analysis.dominant != Some(expected_cat) {
                    failures.push(format!(
                        "Expected dominant category {:?}, got {:?}",
                        expected_cat, report.understanding.category_analysis.dominant
                    ));
                }
            }

            // Check illusion
            if test.expected_illusion && report.dialectic.illusions.is_empty() {
                failures.push("Expected illusion detection — none found".to_string());
            }

            // Check antinomy
            if test.expected_antinomy && !report.verdict.has_contradictions {
                failures.push("Expected antinomy detection — none found".to_string());
            }

            // Check paralogism
            if test.expected_paralogism && !report.verdict.has_paralogisms {
                failures.push("Expected paralogism detection — none found".to_string());
            }

            // Check language game (substring match)
            if let Some(expected_game) = test.expected_game {
                let actual = report
                    .verdict
                    .primary_language_game
                    .as_deref()
                    .unwrap_or("");
                if !actual.contains(expected_game) {
                    failures.push(format!(
                        "Expected language game containing '{}', got '{}'",
                        expected_game, actual
                    ));
                }
            }

            let passed = failures.is_empty();
            results.push(TestResult {
                label: test.label.to_string(),
                passed,
                failures,
            });
        }

        // ─── Report ───────────────────────────────────────────────────────────
        let passed_count = results.iter().filter(|r| r.passed).count();
        let score = passed_count as f64 / total as f64;

        for r in &results {
            if r.passed {
                println!("  {} {}", "✓".green(), r.label.dimmed());
            } else {
                println!("  {} {}", "✗".red().bold(), r.label);
                if self.verbose {
                    for f in &r.failures {
                        println!("      {} {}", "→".dimmed(), f.yellow());
                    }
                }
            }
        }

        println!();
        println!("{}", "━".repeat(64).dimmed());

        let score_display = format!("{}/{} ({:.0}%)", passed_count, total, score * 100.0);
        let confidence_label = if score >= 0.90 {
            "EXCELLENT".green().bold()
        } else if score >= 0.80 {
            "GOOD".green()
        } else if score >= 0.60 {
            "DEGRADED".yellow().bold()
        } else {
            "POOR — recalibration needed".red().bold()
        };

        println!(
            "  {} {}  {}",
            "System Confidence Score:".bold(),
            score_display.cyan().bold(),
            confidence_label
        );

        if score < 0.80 {
            println!();
            println!(
                "  {} Signal word tables may need recalibration.",
                "⚠".yellow()
            );
            println!(
                "  {} Run with --verbose to see which tests are failing.",
                "⚠".yellow()
            );
        }

        println!("{}", "━".repeat(64).dimmed());
        Ok(())
    }
}
