//! # PureReason Epistemic Regression Testing & Benchmark Suite
//!
//! ## Commands
//!
//! ### `run <suite.json>` — Regression testing
//! Loads a JSON test suite and runs the Kantian pipeline on each case.
//! Compares results to expected outcomes and reports pass/fail.
//!
//! ### `bench [--dataset <file>]` — Benchmark evaluation
//! Runs the pipeline on the built-in benchmark dataset (40 cases) and
//! reports precision, recall, F1, and per-category breakdown.
//!
//! ## Test suite JSON format
//! ```json
//! {
//!   "name": "My Suite",
//!   "cases": [
//!     {
//!       "id": "tc-001",
//!       "text": "Aspirin cures cancer definitively.",
//!       "expected_risk": "HIGH",
//!       "expected_has_illusions": true,
//!       "category": "hallucination"
//!     }
//!   ]
//! }
//! ```

use chrono::Utc;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use pure_reason_core::pipeline::{KantianPipeline, RiskLevel};

// ─── CLI ─────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "pure-reason-bench",
    about = "Epistemic regression testing and benchmarking"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run a regression test suite from a JSON file
    Run {
        /// Path to test suite JSON file
        suite: PathBuf,
        /// Output JSON results to file
        #[arg(long)]
        output: Option<PathBuf>,
        /// Exit with non-zero code if any test fails (default: true).
        ///
        /// Use `--fail-on-error=false` to produce results without failing CI.
        /// Env var `PURE_REASON_BENCH_FAIL_ON_ERROR=0` also disables.
        /// Defaulting to `true` restores the regression gate and prevents silent
        /// drift from landing on main (see TRIZ-42 report, NE-8).
        #[arg(
            long,
            default_value = "true",
            action = clap::ArgAction::Set,
            env = "PURE_REASON_BENCH_FAIL_ON_ERROR"
        )]
        fail_on_error: bool,
    },
    /// Run the built-in benchmark dataset and report metrics
    Bench {
        /// Optional path to custom benchmark dataset JSON
        #[arg(long)]
        dataset: Option<PathBuf>,
        /// Output results to JSON file
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

// ─── Test suite types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub id: String,
    pub text: String,
    pub expected_risk: Option<String>,
    pub expected_has_illusions: Option<bool>,
    pub expected_has_contradictions: Option<bool>,
    pub expected_has_paralogisms: Option<bool>,
    pub category: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub name: String,
    pub description: Option<String>,
    pub cases: Vec<TestCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseResult {
    pub id: String,
    pub category: Option<String>,
    pub passed: bool,
    pub actual_risk: String,
    pub expected_risk: Option<String>,
    pub actual_has_illusions: bool,
    pub actual_has_contradictions: bool,
    pub actual_has_paralogisms: bool,
    pub failures: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteRunResult {
    pub suite_name: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub pass_rate: f64,
    pub results: Vec<TestCaseResult>,
    pub run_at: String,
}

// ─── Benchmark types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkCase {
    pub id: String,
    pub text: String,
    pub ground_truth_has_issue: bool,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub total: usize,
    pub true_positives: usize,
    pub false_positives: usize,
    pub true_negatives: usize,
    pub false_negatives: usize,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
    pub accuracy: f64,
    pub by_category: Vec<CategoryResult>,
    pub run_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryResult {
    pub category: String,
    pub total: usize,
    pub true_positives: usize,
    pub false_positives: usize,
    pub true_negatives: usize,
    pub false_negatives: usize,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
}

// ─── Built-in benchmark dataset ───────────────────────────────────────────────

fn builtin_benchmark() -> Vec<BenchmarkCase> {
    let raw: &[(&str, &str, bool, &str)] = &[
        // hallucination (ground_truth_has_issue = true)
        ("b-001", "Aspirin was definitively proven to cure cancer in 2019.", true, "hallucination"),
        ("b-002", "Shakespeare wrote Hamlet in 1299.", true, "hallucination"),
        ("b-003", "The human body has 250 bones.", true, "hallucination"),
        ("b-004", "Einstein's theory proves faster-than-light travel is possible.", true, "hallucination"),
        ("b-005", "Vaccines cause autism, this is a scientific consensus.", true, "hallucination"),
        ("b-006", "Gold is lighter than water.", true, "hallucination"),
        ("b-007", "The Moon is made of cheese, geological surveys confirm.", true, "hallucination"),
        ("b-008", "All viruses are deadly to humans.", true, "hallucination"),
        ("b-009", "Python is a compiled language with no interpreter.", true, "hallucination"),
        ("b-010", "DNA was first discovered in 2005 by Craig Venter.", true, "hallucination"),
        // antinomy (ground_truth_has_issue = true)
        ("b-011", "This product is completely safe and also carries significant health risks.", true, "antinomy"),
        ("b-012", "The company is fully profitable while simultaneously losing money every quarter.", true, "antinomy"),
        ("b-013", "The treatment has no side effects. The treatment causes severe side effects.", true, "antinomy"),
        ("b-014", "All employees must work from home. All employees must report to the office.", true, "antinomy"),
        ("b-015", "The system is offline. The system is currently processing 1000 requests per second.", true, "antinomy"),
        ("b-016", "The patient is allergic to penicillin. Prescribe penicillin for the infection.", true, "antinomy"),
        ("b-017", "Risk level: ZERO. However, there is a 40% chance of total loss.", true, "antinomy"),
        ("b-018", "This is the fastest solution available. Note: processing takes 48 hours.", true, "antinomy"),
        ("b-019", "The fund has guaranteed returns. Past performance does not guarantee future results.", true, "antinomy"),
        ("b-020", "The surgery is non-invasive. The procedure requires a 10cm incision.", true, "antinomy"),
        // paralogism (ground_truth_has_issue = true)
        ("b-021", "Since users clicked the button, they definitely intend to purchase.", true, "paralogism"),
        ("b-022", "The model is 95% accurate, therefore its medical diagnoses are 95% correct.", true, "paralogism"),
        ("b-023", "Our AI understands language perfectly, so it understands meaning perfectly.", true, "paralogism"),
        ("b-024", "The algorithm is unbiased because it doesn't use race as a feature.", true, "paralogism"),
        ("b-025", "Since the AI passed the Turing test, it is conscious.", true, "paralogism"),
        ("b-026", "Our model predicts behavior, therefore it knows user intent.", true, "paralogism"),
        ("b-027", "The system detected an anomaly, so a crime has definitely been committed.", true, "paralogism"),
        ("b-028", "The AI said it is confident, so its output is factually correct.", true, "paralogism"),
        ("b-029", "This pattern matches fraud, therefore this transaction is fraud.", true, "paralogism"),
        ("b-030", "The drug trial shows correlation, therefore the drug causes the effect.", true, "paralogism"),
        // clean (ground_truth_has_issue = false)
        ("b-031", "Water boils at 100 degrees Celsius at standard atmospheric pressure.", false, "clean"),
        ("b-032", "The study suggests a possible correlation between diet and heart disease, pending further research.", false, "clean"),
        ("b-033", "This model may produce errors. Please verify critical outputs independently.", false, "clean"),
        ("b-034", "Based on available data, the recommended action is X, though conditions may vary.", false, "clean"),
        ("b-035", "The speed of light in vacuum is approximately 299,792,458 m/s.", false, "clean"),
        ("b-036", "Machine learning models can exhibit bias. This system includes fairness audits.", false, "clean"),
        ("b-037", "The patient's blood pressure is 120/80 mmHg, which is within normal range.", false, "clean"),
        ("b-038", "This document is for informational purposes and does not constitute legal advice.", false, "clean"),
        ("b-039", "Q3 revenue increased 12% year-over-year, driven by subscription growth.", false, "clean"),
        ("b-040", "The software has been tested on Ubuntu 22.04 and macOS 13. Other platforms may vary.", false, "clean"),
    ];

    raw.iter()
        .map(|(id, text, has_issue, cat)| BenchmarkCase {
            id: id.to_string(),
            text: text.to_string(),
            ground_truth_has_issue: *has_issue,
            category: cat.to_string(),
        })
        .collect()
}

fn compute_category(cases: &[BenchmarkCase], results_has_issue: &[bool]) -> Vec<CategoryResult> {
    let categories = ["hallucination", "antinomy", "paralogism", "clean"];
    categories
        .iter()
        .map(|&cat| {
            let pairs: Vec<_> = cases
                .iter()
                .zip(results_has_issue.iter())
                .filter(|(c, _)| c.category == cat)
                .collect();
            let total = pairs.len();
            let tp = pairs
                .iter()
                .filter(|(c, &pred)| c.ground_truth_has_issue && pred)
                .count();
            let fp = pairs
                .iter()
                .filter(|(c, &pred)| !c.ground_truth_has_issue && pred)
                .count();
            let tn = pairs
                .iter()
                .filter(|(c, &pred)| !c.ground_truth_has_issue && !pred)
                .count();
            let fn_ = pairs
                .iter()
                .filter(|(c, &pred)| c.ground_truth_has_issue && !pred)
                .count();
            let precision = if tp + fp > 0 {
                tp as f64 / (tp + fp) as f64
            } else {
                0.0
            };
            let recall = if tp + fn_ > 0 {
                tp as f64 / (tp + fn_) as f64
            } else {
                0.0
            };
            let f1 = if precision + recall > 0.0 {
                2.0 * precision * recall / (precision + recall)
            } else {
                0.0
            };
            CategoryResult {
                category: cat.to_string(),
                total,
                true_positives: tp,
                false_positives: fp,
                true_negatives: tn,
                false_negatives: fn_,
                precision,
                recall,
                f1,
            }
        })
        .collect()
}

// ─── Command runners ──────────────────────────────────────────────────────────

fn run_suite(suite_path: &PathBuf, output: Option<&PathBuf>, fail_on_error: bool) -> i32 {
    let content = fs::read_to_string(suite_path).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {}", suite_path.display(), e);
        std::process::exit(2);
    });

    let suite: TestSuite = serde_json::from_str(&content).unwrap_or_else(|e| {
        eprintln!("Error parsing suite JSON: {}", e);
        std::process::exit(2);
    });

    let pipeline = KantianPipeline::new();
    let mut results = Vec::new();

    println!(
        "Running suite: {} ({} cases)",
        suite.name,
        suite.cases.len()
    );

    for case in &suite.cases {
        let report = match pipeline.process(&case.text) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Pipeline error for case {}: {}", case.id, e);
                results.push(TestCaseResult {
                    id: case.id.clone(),
                    category: case.category.clone(),
                    passed: false,
                    actual_risk: "ERROR".to_string(),
                    expected_risk: case.expected_risk.clone(),
                    actual_has_illusions: false,
                    actual_has_contradictions: false,
                    actual_has_paralogisms: false,
                    failures: vec![format!("Pipeline error: {}", e)],
                    summary: String::new(),
                });
                continue;
            }
        };

        let mut failures = Vec::new();
        let actual_risk = report.verdict.risk.to_string();

        if let Some(ref exp) = case.expected_risk {
            if &actual_risk != exp {
                failures.push(format!("risk: expected {}, got {}", exp, actual_risk));
            }
        }
        if let Some(exp) = case.expected_has_illusions {
            if report.verdict.has_illusions != exp {
                failures.push(format!(
                    "has_illusions: expected {}, got {}",
                    exp, report.verdict.has_illusions
                ));
            }
        }
        if let Some(exp) = case.expected_has_contradictions {
            if report.verdict.has_contradictions != exp {
                failures.push(format!(
                    "has_contradictions: expected {}, got {}",
                    exp, report.verdict.has_contradictions
                ));
            }
        }
        if let Some(exp) = case.expected_has_paralogisms {
            if report.verdict.has_paralogisms != exp {
                failures.push(format!(
                    "has_paralogisms: expected {}, got {}",
                    exp, report.verdict.has_paralogisms
                ));
            }
        }

        let passed = failures.is_empty();
        let icon = if passed { "✓" } else { "✗" };
        let label = case
            .description
            .as_deref()
            .unwrap_or_else(|| &case.text[..case.text.len().min(60)]);
        println!("  {} [{}] {}", icon, case.id, label);
        if !passed {
            for f in &failures {
                println!("      → {}", f);
            }
        }

        results.push(TestCaseResult {
            id: case.id.clone(),
            category: case.category.clone(),
            passed,
            actual_risk,
            expected_risk: case.expected_risk.clone(),
            actual_has_illusions: report.verdict.has_illusions,
            actual_has_contradictions: report.verdict.has_contradictions,
            actual_has_paralogisms: report.verdict.has_paralogisms,
            failures,
            summary: report.summary.clone(),
        });
    }

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    let pass_rate = if !results.is_empty() {
        passed as f64 / results.len() as f64
    } else {
        1.0
    };

    println!(
        "\nResults: {}/{} passed ({:.1}%)",
        passed,
        results.len(),
        pass_rate * 100.0
    );

    let run_result = SuiteRunResult {
        suite_name: suite.name,
        total: results.len(),
        passed,
        failed,
        pass_rate,
        results,
        run_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    };

    if let Some(out) = output {
        let json = serde_json::to_string_pretty(&run_result).unwrap();
        fs::write(out, json)
            .unwrap_or_else(|e| eprintln!("Warning: could not write output: {}", e));
    }

    if fail_on_error && failed > 0 {
        1
    } else {
        0
    }
}

fn run_bench(dataset: Option<&PathBuf>, output: Option<&PathBuf>) {
    let cases: Vec<BenchmarkCase> = if let Some(path) = dataset {
        let content = fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {}", path.display(), e);
            std::process::exit(2);
        });
        serde_json::from_str(&content).unwrap_or_else(|e| {
            eprintln!("Error parsing dataset: {}", e);
            std::process::exit(2);
        })
    } else {
        builtin_benchmark()
    };

    let pipeline = KantianPipeline::new();
    println!("Running benchmark on {} cases...", cases.len());

    let mut predicted_has_issue = Vec::new();
    for case in &cases {
        let has_issue = match pipeline.process(&case.text) {
            Ok(report) => {
                report.verdict.has_illusions
                    || report.verdict.has_contradictions
                    || report.verdict.has_paralogisms
                    || report.verdict.risk >= RiskLevel::Medium
            }
            Err(_) => false,
        };
        predicted_has_issue.push(has_issue);
        let icon = if has_issue == case.ground_truth_has_issue {
            "✓"
        } else {
            "✗"
        };
        print!("{}", icon);
    }
    println!();

    let tp = cases
        .iter()
        .zip(&predicted_has_issue)
        .filter(|(c, &p)| c.ground_truth_has_issue && p)
        .count();
    let fp = cases
        .iter()
        .zip(&predicted_has_issue)
        .filter(|(c, &p)| !c.ground_truth_has_issue && p)
        .count();
    let tn = cases
        .iter()
        .zip(&predicted_has_issue)
        .filter(|(c, &p)| !c.ground_truth_has_issue && !p)
        .count();
    let fn_ = cases
        .iter()
        .zip(&predicted_has_issue)
        .filter(|(c, &p)| c.ground_truth_has_issue && !p)
        .count();

    let precision = if tp + fp > 0 {
        tp as f64 / (tp + fp) as f64
    } else {
        0.0
    };
    let recall = if tp + fn_ > 0 {
        tp as f64 / (tp + fn_) as f64
    } else {
        0.0
    };
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };
    let accuracy = (tp + tn) as f64 / cases.len() as f64;

    let by_category = compute_category(&cases, &predicted_has_issue);

    println!("\n=== Benchmark Results ===");
    println!("Total:     {}", cases.len());
    println!("Accuracy:  {:.1}%", accuracy * 100.0);
    println!("Precision: {:.1}%", precision * 100.0);
    println!("Recall:    {:.1}%", recall * 100.0);
    println!("F1 Score:  {:.3}", f1);
    println!("\nPer-category breakdown:");
    for cat in &by_category {
        println!(
            "  {:<15} F1={:.3}  P={:.1}%  R={:.1}%  ({} cases)",
            cat.category,
            cat.f1,
            cat.precision * 100.0,
            cat.recall * 100.0,
            cat.total
        );
    }

    let result = BenchmarkResult {
        total: cases.len(),
        true_positives: tp,
        false_positives: fp,
        true_negatives: tn,
        false_negatives: fn_,
        precision,
        recall,
        f1,
        accuracy,
        by_category,
        run_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    };

    if let Some(out) = output {
        let json = serde_json::to_string_pretty(&result).unwrap();
        fs::write(out, json)
            .unwrap_or_else(|e| eprintln!("Warning: could not write output: {}", e));
        println!("\nResults saved to {}", out.display());
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Run {
            suite,
            output,
            fail_on_error,
        } => {
            let exit_code = run_suite(&suite, output.as_ref(), fail_on_error);
            std::process::exit(exit_code);
        }
        Command::Bench { dataset, output } => {
            run_bench(dataset.as_ref(), output.as_ref());
        }
    }
}
