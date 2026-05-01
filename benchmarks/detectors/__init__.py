"""benchmarks.detectors — hallucination detector subpackage.

Re-exports all public benchmark functions and the core error type.
"""

from .core import BenchmarkExecutionError
from .felm_bench import benchmark_felm
from .hallulens import benchmark_hallulens
from .hallumix import benchmark_hallumix
from .halueval import benchmark_halueval_dialogue, benchmark_halueval_qa
from .logicbench import benchmark_logicbench
from .metrics import compute_metrics
from .ragtruth import benchmark_faithbench, benchmark_ragtruth
from .truthfulqa import benchmark_truthfulqa

__all__ = [
    "BenchmarkExecutionError",
    "benchmark_faithbench",
    "benchmark_felm",
    "benchmark_hallulens",
    "benchmark_hallumix",
    "benchmark_halueval_dialogue",
    "benchmark_halueval_qa",
    "benchmark_logicbench",
    "benchmark_ragtruth",
    "benchmark_truthfulqa",
    "compute_metrics",
]
