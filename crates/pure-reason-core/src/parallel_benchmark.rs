//! # Parallel Benchmark Executor: Async/concurrent benchmarking via tokio
//!
//! Executes 7 benchmark suites in parallel instead of sequentially.
//! Estimated speedup: 15 hours → 2-3 hours (5x).
//!
//! Benchmarks:
//! - BIG-Bench: 650 tasks
//! - MMLU-Pro: 12,000 tasks
//! - ARC: 7,800 tasks
//! - GSM8K: 850 tasks
//! - HumanEval: 164 tasks
//! - DROP: 10,000 tasks
//! - MATH: 12,500 tasks
//! **Total**: 44,000+ tasks

use crate::benchmark_integration::{BenchmarkMetrics, BenchmarkTask, TaskResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Benchmark suite identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BenchmarkId {
    BigBench,
    MmluPro,
    Arc,
    Gsm8k,
    HumanEval,
    Drop,
    Math,
}

impl BenchmarkId {
    pub fn name(&self) -> &'static str {
        match self {
            Self::BigBench => "BIG-Bench",
            Self::MmluPro => "MMLU-Pro",
            Self::Arc => "ARC",
            Self::Gsm8k => "GSM8K",
            Self::HumanEval => "HumanEval",
            Self::Drop => "DROP",
            Self::Math => "MATH",
        }
    }

    pub fn task_count(&self) -> usize {
        match self {
            Self::BigBench => 650,
            Self::MmluPro => 12_000,
            Self::Arc => 7_800,
            Self::Gsm8k => 850,
            Self::HumanEval => 164,
            Self::Drop => 10_000,
            Self::Math => 12_500,
        }
    }

    pub fn all() -> &'static [BenchmarkId] {
        &[
            Self::BigBench,
            Self::MmluPro,
            Self::Arc,
            Self::Gsm8k,
            Self::HumanEval,
            Self::Drop,
            Self::Math,
        ]
    }
}

/// Result of parallel benchmark execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelBenchmarkResult {
    /// Results per benchmark
    pub benchmarks: HashMap<String, BenchmarkMetrics>,
    /// Total tasks across all benchmarks
    pub total_tasks: usize,
    /// Total correct across all benchmarks
    pub total_correct: usize,
    /// Macro F1 (average F1 across benchmarks)
    pub macro_f1: f64,
    /// Weighted F1 (tasks-weighted)
    pub weighted_f1: f64,
    /// Total time in seconds
    pub total_time_secs: f64,
    /// Timestamp
    pub timestamp: u64,
}

impl ParallelBenchmarkResult {
    /// Compute from per-benchmark metrics
    pub fn from_metrics(metrics: HashMap<String, BenchmarkMetrics>, total_time_secs: f64) -> Self {
        let mut total_tasks = 0;
        let mut total_correct = 0;
        let mut f1_sum = 0.0;
        let mut task_weighted_f1 = 0.0;

        for m in metrics.values() {
            total_tasks += m.total;
            total_correct += m.correct;
            f1_sum += m.f1;
            task_weighted_f1 += m.f1 * m.total as f64;
        }

        let count = metrics.len() as f64;
        let macro_f1 = if count > 0.0 { f1_sum / count } else { 0.0 };
        let weighted_f1 = if total_tasks > 0 {
            task_weighted_f1 / total_tasks as f64
        } else {
            0.0
        };

        Self {
            benchmarks: metrics,
            total_tasks,
            total_correct,
            macro_f1,
            weighted_f1,
            total_time_secs,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Benchmark runner
#[derive(Debug, Clone)]
pub struct BenchmarkRunner {
    /// Deterministic seed for reproducibility
    pub seed: u64,
}

impl BenchmarkRunner {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Generate synthetic tasks for a benchmark (for Phase 3 testing)
    pub fn generate_tasks(&self, benchmark_id: BenchmarkId) -> Vec<BenchmarkTask> {
        let mut tasks = Vec::new();
        let count = benchmark_id.task_count();

        for i in 0..count {
            let task = BenchmarkTask {
                id: format!("{}-{}", benchmark_id.name(), i),
                benchmark: benchmark_id.name().to_lowercase(),
                domain: self.infer_domain(benchmark_id),
                claim: format!(
                    "Sample task {} for {} (seed={})",
                    i,
                    benchmark_id.name(),
                    self.seed
                ),
                ground_truth: format!("answer_{}", i % 5), // Deterministic for reproducibility
                difficulty: match i % 3 {
                    0 => "easy".to_string(),
                    1 => "medium".to_string(),
                    _ => "hard".to_string(),
                },
            };
            tasks.push(task);
        }

        tasks
    }

    /// Infer domain from benchmark type
    fn infer_domain(&self, benchmark_id: BenchmarkId) -> String {
        match benchmark_id {
            BenchmarkId::BigBench => "reasoning".to_string(),
            BenchmarkId::MmluPro => "knowledge".to_string(),
            BenchmarkId::Arc => "reasoning".to_string(),
            BenchmarkId::Gsm8k => "math".to_string(),
            BenchmarkId::HumanEval => "code".to_string(),
            BenchmarkId::Drop => "reading".to_string(),
            BenchmarkId::Math => "math".to_string(),
        }
    }

    /// Simulate evaluating a task (returns synthetic result for Phase 3 testing)
    pub fn evaluate_task(&self, task: &BenchmarkTask) -> TaskResult {
        // Deterministic scoring based on seed and task id hash
        let hash = deterministic_hash(&format!("{}{}", self.seed, task.id));
        let correct = (hash % 100) > 15; // ~85% baseline accuracy
        let confidence = 0.5 + ((hash % 50) as f64 / 100.0); // 0.5-1.0

        TaskResult {
            task_id: task.id.clone(),
            prediction: if correct {
                task.ground_truth.clone()
            } else {
                format!("wrong_{}", hash % 4)
            },
            correct,
            confidence,
            active_phase: task.domain.clone(),
            latency_ms: 50.0 + ((hash % 100) as f64),
        }
    }
}

/// Deterministic hash function for reproducibility
fn deterministic_hash(s: &str) -> u64 {
    let mut hash = 0u64;
    for byte in s.as_bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(*byte as u64);
    }
    hash
}

/// Parallel benchmark executor
pub struct ParallelBenchmarkExecutor {
    runner: Arc<BenchmarkRunner>,
}

impl ParallelBenchmarkExecutor {
    pub fn new(seed: u64) -> Self {
        Self {
            runner: Arc::new(BenchmarkRunner::new(seed)),
        }
    }

    /// Execute all 7 benchmarks in parallel
    pub async fn run_all_benchmarks(&self) -> Result<ParallelBenchmarkResult, String> {
        let start = std::time::Instant::now();
        let results = Arc::new(Mutex::new(HashMap::new()));

        let mut tasks = vec![];

        for benchmark_id in BenchmarkId::all() {
            let runner = Arc::clone(&self.runner);
            let results_ref = Arc::clone(&results);
            let benchmark_id = *benchmark_id;

            let task = tokio::spawn(async move {
                Self::run_single_benchmark(runner, results_ref, benchmark_id).await
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        for task in tasks {
            task.await
                .map_err(|e| e.to_string())?
                .map_err(|e| e.to_string())?;
        }

        let elapsed = start.elapsed().as_secs_f64();
        let metrics: HashMap<String, BenchmarkMetrics> =
            Arc::try_unwrap(results).unwrap().into_inner();

        Ok(ParallelBenchmarkResult::from_metrics(metrics, elapsed))
    }

    async fn run_single_benchmark(
        runner: Arc<BenchmarkRunner>,
        results: Arc<Mutex<HashMap<String, BenchmarkMetrics>>>,
        benchmark_id: BenchmarkId,
    ) -> Result<(), String> {
        // Generate tasks
        let tasks = runner.generate_tasks(benchmark_id);

        // Evaluate all tasks for this benchmark
        let mut eval_results = vec![];
        for task in tasks {
            let result = runner.evaluate_task(&task);
            eval_results.push(result);
        }

        // Compute metrics
        let metrics =
            BenchmarkMetrics::from_results(benchmark_id.name().to_string(), &eval_results);

        // Store results
        {
            let mut store = results.lock().await;
            store.insert(benchmark_id.name().to_string(), metrics);
        }

        Ok(())
    }

    /// Run subset of benchmarks (for testing)
    pub async fn run_benchmarks(
        &self,
        benchmark_ids: &[BenchmarkId],
    ) -> Result<ParallelBenchmarkResult, String> {
        let start = std::time::Instant::now();
        let results = Arc::new(Mutex::new(HashMap::new()));

        let mut tasks = vec![];

        for benchmark_id in benchmark_ids {
            let runner = Arc::clone(&self.runner);
            let results_ref = Arc::clone(&results);
            let benchmark_id = *benchmark_id;

            let task = tokio::spawn(async move {
                Self::run_single_benchmark(runner, results_ref, benchmark_id).await
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        for task in tasks {
            task.await
                .map_err(|e| e.to_string())?
                .map_err(|e| e.to_string())?;
        }

        let elapsed = start.elapsed().as_secs_f64();
        let metrics: HashMap<String, BenchmarkMetrics> =
            Arc::try_unwrap(results).unwrap().into_inner();

        Ok(ParallelBenchmarkResult::from_metrics(metrics, elapsed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_id_names() {
        assert_eq!(BenchmarkId::BigBench.name(), "BIG-Bench");
        assert_eq!(BenchmarkId::MmluPro.name(), "MMLU-Pro");
        assert_eq!(BenchmarkId::Gsm8k.name(), "GSM8K");
    }

    #[test]
    fn test_benchmark_id_task_counts() {
        assert_eq!(BenchmarkId::BigBench.task_count(), 650);
        assert_eq!(BenchmarkId::MmluPro.task_count(), 12_000);
        let total: usize = BenchmarkId::all().iter().map(|b| b.task_count()).sum();
        assert_eq!(total, 43_964);
    }

    #[test]
    fn test_benchmark_runner_generation() {
        let runner = BenchmarkRunner::new(42);
        let tasks = runner.generate_tasks(BenchmarkId::Gsm8k);
        assert_eq!(tasks.len(), 850);
        assert_eq!(tasks[0].benchmark, "gsm8k");
        assert_eq!(tasks[0].domain, "math");
    }

    #[test]
    fn test_benchmark_runner_determinism() {
        let runner = BenchmarkRunner::new(42);
        let task = runner.generate_tasks(BenchmarkId::Gsm8k)[0].clone();

        let result1 = runner.evaluate_task(&task);
        let result2 = runner.evaluate_task(&task);

        assert_eq!(result1.correct, result2.correct);
        assert_eq!(result1.confidence, result2.confidence);
    }

    #[tokio::test]
    async fn test_parallel_executor_subset() {
        let executor = ParallelBenchmarkExecutor::new(42);
        let result = executor
            .run_benchmarks(&[BenchmarkId::Gsm8k, BenchmarkId::HumanEval])
            .await
            .expect("benchmarks should run");

        assert_eq!(result.benchmarks.len(), 2);
        assert!(result.total_tasks > 0);
        assert!(result.total_time_secs > 0.0);
    }

    #[tokio::test]
    async fn test_parallel_benchmark_result() {
        let executor = ParallelBenchmarkExecutor::new(42);
        let result = executor
            .run_benchmarks(&[BenchmarkId::Gsm8k])
            .await
            .expect("benchmark should run");

        assert!(result.macro_f1 >= 0.0 && result.macro_f1 <= 1.0);
        assert!(result.weighted_f1 >= 0.0 && result.weighted_f1 <= 1.0);
        assert!(result.total_time_secs > 0.0);
        assert!(result.timestamp > 0);
    }

    #[tokio::test]
    async fn test_all_benchmarks_parallel() {
        let executor = ParallelBenchmarkExecutor::new(42);
        let result = executor
            .run_all_benchmarks()
            .await
            .expect("all benchmarks should run");

        // Should have 7 benchmarks
        assert_eq!(result.benchmarks.len(), 7);

        // Total tasks should match expected
        assert_eq!(result.total_tasks, 43_964);

        // Metrics should be valid
        assert!(result.macro_f1 > 0.0);
        assert!(result.weighted_f1 > 0.0);
        assert!(result.total_time_secs > 0.0);

        // Verify all benchmark names are present
        assert!(result.benchmarks.contains_key("BIG-Bench"));
        assert!(result.benchmarks.contains_key("MMLU-Pro"));
        assert!(result.benchmarks.contains_key("GSM8K"));
        assert!(result.benchmarks.contains_key("HumanEval"));
    }
}
