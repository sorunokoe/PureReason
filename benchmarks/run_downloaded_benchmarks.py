#!/usr/bin/env python3
"""
Run PureReason against downloaded official benchmark files.

This script evaluates the heuristic PureReason pipeline on benchmark assets fetched
via `benchmarks/download_benchmarks.py`. It expands beyond the legacy local snapshots
to include RAGTruth, FaithBench, and FELM.
"""

import argparse
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
BENCHMARKS_DIR = REPO / "benchmarks"
DOWNLOADS_DIR = BENCHMARKS_DIR / "downloads"
RESULTS_DIR = BENCHMARKS_DIR / "results"
RESULTS_DIR.mkdir(exist_ok=True)

sys.path.insert(0, str(REPO))

from benchmarks.detectors import (  # noqa: E402
    benchmark_faithbench,
    benchmark_felm,
    benchmark_hallulens,
    benchmark_hallumix,
    benchmark_halueval_dialogue,
    benchmark_halueval_qa,
    benchmark_logicbench,
    benchmark_ragtruth,
    benchmark_truthfulqa,
)
from benchmarks.detectors import core as _det_core  # noqa: E402
from benchmarks.detectors.core import ensure_cli  # noqa: E402

BENCHMARKS = {
    "truthfulqa": benchmark_truthfulqa,
    "halueval_qa": benchmark_halueval_qa,
    "halueval_dialogue": benchmark_halueval_dialogue,
    "ragtruth": benchmark_ragtruth,
    "faithbench": benchmark_faithbench,
    "felm": benchmark_felm,
    "hallumix": benchmark_hallumix,
    "hallulens": benchmark_hallulens,
    "logicbench": benchmark_logicbench,
}

DOWNLOAD_REQUIREMENTS = {
    "truthfulqa": ["truthfulqa/TruthfulQA.csv"],
    "halueval_qa": ["halueval/qa_data.json"],
    "halueval_dialogue": ["halueval/dialogue_data.json"],
    "ragtruth": ["ragtruth/source_info.jsonl", "ragtruth/response.jsonl"],
    "faithbench": ["faithbench/FaithBench.csv"],
    "felm": ["felm/all.jsonl"],
    "hallumix": ["hallumix/train.parquet"],
    "hallulens": ["hallulens/precise_wiki_test.parquet"],
    "logicbench": ["logicbench/modus_tollens.json"],
}

DOWNLOAD_ALIASES = {
    "truthfulqa": "truthfulqa",
    "halueval_qa": "halueval",
    "halueval_dialogue": "halueval",
    "ragtruth": "ragtruth",
    "faithbench": "faithbench",
    "felm": "felm",
    "hallumix": "hallumix",
    "hallulens": "hallulens",
    "logicbench": "logicbench",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run PureReason on downloaded official benchmark datasets."
    )
    parser.add_argument(
        "--benchmarks",
        default="all",
        help=(
            "Comma-separated benchmark keys to run "
            "(truthfulqa,halueval_qa,halueval_dialogue,ragtruth,faithbench,felm) "
            "or 'all'."
        ),
    )
    parser.add_argument(
        "--downloads-dir",
        default=str(DOWNLOADS_DIR),
        help="Directory containing downloaded benchmark assets.",
    )
    parser.add_argument(
        "--n",
        type=int,
        default=50,
        help="Balanced sample size per class for each benchmark.",
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=42,
        help="Sampling seed.",
    )
    parser.add_argument(
        "--workers",
        type=int,
        default=8,
        help="Parallel workers for PureReason CLI evaluation.",
    )
    parser.add_argument(
        "--output",
        default=str(RESULTS_DIR / "SUMMARY_downloaded_heuristic.json"),
        help="Path to write the JSON summary.",
    )
    parser.add_argument(
        "--holdout",
        action="store_true",
        default=False,
        help=(
            "S53 holdout protocol: evaluate on a second independent draw "
            "(samples n+1…2n), never used during threshold calibration. "
            "Eliminates test-set leakage and provides unbiased F1 estimates."
        ),
    )
    return parser.parse_args()


def resolve_selection(selection: str) -> list[str]:
    if selection == "all":
        return list(BENCHMARKS)
    selected = [item.strip() for item in selection.split(",") if item.strip()]
    unknown = [item for item in selected if item not in BENCHMARKS]
    if unknown:
        raise SystemExit(
            "Unknown benchmark(s): "
            + ", ".join(sorted(unknown))
            + f". Available: {', '.join(BENCHMARKS)}"
        )
    return selected


def main() -> int:
    args = parse_args()
    _det_core._HOLDOUT_MODE = args.holdout
    ensure_cli()
    downloads_dir = Path(args.downloads_dir).resolve()
    selected = resolve_selection(args.benchmarks)

    missing = []
    for key in selected:
        required = DOWNLOAD_REQUIREMENTS[key]
        if any(not (downloads_dir / relative_path).exists() for relative_path in required):
            missing.append(key)
    if missing:
        downloader_keys = sorted({DOWNLOAD_ALIASES[key] for key in missing})
        raise SystemExit(
            "Missing downloaded assets for: "
            + ", ".join(missing)
            + ". Run `python3 benchmarks/download_benchmarks.py --benchmarks "
            + ",".join(downloader_keys)
            + "` first."
        )

    mode_tag = " [S53 HOLDOUT — unbiased estimates]" if args.holdout else ""
    print("PureReason Official Benchmark Evaluation")
    print("=======================================")
    print(f"Downloads:  {downloads_dir}")
    print(f"Benchmarks: {', '.join(selected)}")
    print(f"Mode:       Heuristic (pure-reason analyze){mode_tag}")
    print(f"n/class:    {args.n}")

    summary = {
        "version": "downloaded_heuristic",
        "holdout": args.holdout,
        "downloads_dir": str(downloads_dir),
        "sample_size_per_class": args.n,
        "benchmarks": {},
    }
    for key in selected:
        result = BENCHMARKS[key](downloads_dir, args.n, args.seed, args.workers)
        summary["benchmarks"][key] = result

    print(f"\n{'=' * 72}\nSUMMARY\n{'=' * 72}")
    print(
        f"{'Benchmark':<22} {'Precision':>10} {'Recall':>8} {'F1':>8} {'95% CI':>16} {'Accuracy':>10}"
    )
    print("-" * 78)
    for key in selected:
        metrics = summary["benchmarks"][key]["metrics"]
        ci = metrics.get("ci_95", {})
        f1_ci = ci.get("f1") or {}
        ci_str = (
            f"[{f1_ci.get('lower', metrics['f1']):.3f}–{f1_ci.get('upper', metrics['f1']):.3f}]"
        )
        print(
            f"{key:<22} {metrics['precision']:>10.3f} {metrics['recall']:>8.3f} "
            f"{metrics['f1']:>8.3f} {ci_str:>16} {metrics['accuracy']:>10.3f}"
        )

    output_path = Path(args.output).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as handle:
        json.dump(summary, handle, indent=2)
    print(f"\nSummary written to {output_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
