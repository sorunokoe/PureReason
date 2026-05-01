#!/usr/bin/env python3
"""
Audit Wikipedia corpus for benchmark leakage.

Checks if corpus articles overlap with benchmark datasets beyond acceptable threshold.

Usage:
    python3 benchmarks/audit_corpus_leak.py \
        --corpus data/corpus/wikipedia_v1.0.jsonl.gz \
        --benchmarks truthfulqa,halueval_qa,ragtruth \
        --threshold 5.0 \
        --output data/corpus/wikipedia_v1.0.audit.json
"""

import argparse
import gzip
import json
from datetime import datetime
from pathlib import Path


def load_corpus_ids(corpus_file: str) -> set[str]:
    """Load article IDs from compressed JSONL corpus."""
    ids = set()

    open_func = gzip.open if corpus_file.endswith(".gz") else open

    with open_func(corpus_file, "rt", encoding="utf-8") as f:
        for line in f:
            record = json.loads(line)
            ids.add(record["id"])

    return ids


def load_benchmark_sources(benchmark_name: str) -> set[str]:
    """
    Load source article IDs/titles from benchmark dataset.

    Returns set of identifiers (Wikipedia IDs or normalized titles).
    """
    # Map benchmark names to their data files
    benchmark_files = {
        "truthfulqa": "benchmarks/data/truthfulqa_questions.jsonl",
        "halueval_qa": "benchmarks/data/halueval_qa_samples.jsonl",
        "ragtruth": "benchmarks/data/ragtruth_samples.jsonl",
        "faithbench": "benchmarks/data/faithbench_samples.jsonl",
        "hallulens": "benchmarks/data/hallulens_samples.jsonl",
    }

    file_path = benchmark_files.get(benchmark_name.lower())
    if not file_path or not Path(file_path).exists():
        print(f"Warning: Benchmark '{benchmark_name}' not found or not supported.")
        return set()

    sources = set()

    with open(file_path, encoding="utf-8") as f:
        for line in f:
            record = json.loads(line)

            # Extract source identifiers (varies by benchmark)
            if "source_url" in record:
                # Extract Wikipedia ID from URL
                url = record["source_url"]
                if "wikipedia.org" in url:
                    wiki_id = url.split("=")[-1] if "=" in url else url.split("/")[-1]
                    sources.add(wiki_id)

            elif "source_title" in record:
                # Use normalized title
                title = record["source_title"].lower().replace(" ", "_")
                sources.add(title)

    return sources


def compute_overlap(corpus_ids: set[str], benchmark_sources: set[str]) -> dict:
    """Compute overlap between corpus and benchmark sources."""
    overlapping = corpus_ids & benchmark_sources

    overlap_percentage = (
        (len(overlapping) / len(benchmark_sources) * 100) if benchmark_sources else 0.0
    )

    return {
        "total_corpus_articles": len(corpus_ids),
        "total_benchmark_sources": len(benchmark_sources),
        "overlapping_articles": len(overlapping),
        "overlap_percentage": overlap_percentage,
        "overlapping_ids": list(overlapping)[:100],  # Sample for debugging
    }


def audit_corpus(corpus_file: str, benchmarks: list[str], threshold: float, output_file: str):
    """
    Audit corpus for benchmark leakage.

    Args:
        corpus_file: Path to Wikipedia corpus (JSONL or JSONL.gz)
        benchmarks: List of benchmark names to check
        threshold: Maximum acceptable overlap percentage (e.g., 5.0 = 5%)
        output_file: Path to write audit report JSON
    """
    print(f"Auditing corpus: {corpus_file}")
    print(f"Benchmarks: {', '.join(benchmarks)}")
    print(f"Threshold: {threshold}%\n")

    # Load corpus IDs
    print("Loading corpus IDs...")
    corpus_ids = load_corpus_ids(corpus_file)
    print(f"Loaded {len(corpus_ids):,} corpus articles.\n")

    # Check each benchmark
    results = []
    max_overlap = 0.0

    for benchmark_name in benchmarks:
        print(f"Checking {benchmark_name}...")

        benchmark_sources = load_benchmark_sources(benchmark_name)

        if not benchmark_sources:
            print("  Skipped (no sources found)\n")
            continue

        overlap_stats = compute_overlap(corpus_ids, benchmark_sources)
        overlap_pct = overlap_stats["overlap_percentage"]

        status = "PASS" if overlap_pct <= threshold else "FAIL"

        result = {
            "name": benchmark_name,
            "total_questions": overlap_stats["total_benchmark_sources"],
            "overlapping_articles": overlap_stats["overlapping_articles"],
            "overlap_percentage": round(overlap_pct, 2),
            "status": status,
        }

        results.append(result)
        max_overlap = max(max_overlap, overlap_pct)

        print(f"  Sources: {overlap_stats['total_benchmark_sources']:,}")
        print(f"  Overlap: {overlap_stats['overlapping_articles']:,} ({overlap_pct:.2f}%)")
        print(f"  Status: {status}\n")

    # Overall status
    overall_status = "PASS" if max_overlap <= threshold else "FAIL"

    # Build audit report
    report = {
        "audit_date": datetime.utcnow().isoformat() + "Z",
        "corpus_file": corpus_file,
        "corpus_version": "1.0",  # TODO: Extract from filename
        "benchmarks_checked": results,
        "overall_status": overall_status,
        "max_overlap_percentage": round(max_overlap, 2),
        "threshold": threshold,
    }

    # Write report
    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2, ensure_ascii=False)

    print(f"Audit report written to: {output_file}")
    print(f"\n{'=' * 60}")
    print(f"OVERALL STATUS: {overall_status}")
    print(f"Max overlap: {max_overlap:.2f}% (threshold: {threshold}%)")
    print(f"{'=' * 60}")

    # Exit with error code if failed
    if overall_status == "FAIL":
        exit(1)


def main():
    parser = argparse.ArgumentParser(description="Audit Wikipedia corpus for benchmark leakage")
    parser.add_argument("--corpus", required=True, help="Path to corpus JSONL (may be gzipped)")
    parser.add_argument("--benchmarks", required=True, help="Comma-separated benchmark names")
    parser.add_argument(
        "--threshold", type=float, default=5.0, help="Max overlap percentage (default: 5.0)"
    )
    parser.add_argument("--output", required=True, help="Path to output audit report JSON")

    args = parser.parse_args()

    benchmarks = [b.strip() for b in args.benchmarks.split(",")]

    audit_corpus(args.corpus, benchmarks, args.threshold, args.output)


if __name__ == "__main__":
    main()
