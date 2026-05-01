"""RAGTruth and FaithBench benchmarks."""

import csv
from pathlib import Path

from .core import load_json_records, text_value, truncate_text
from .metrics import evaluate_pairs_combined, package_result, sample_balanced
from .verdicts import faithbench_combined_verdict, ragtruth_combined_verdict


def benchmark_ragtruth(downloads_dir: Path, n_per_class: int, seed: int, workers: int) -> dict:
    source_path = downloads_dir / "ragtruth" / "source_info.jsonl"
    response_path = downloads_dir / "ragtruth" / "response.jsonl"
    sources = {
        row["source_id"]: row for row in load_json_records(source_path) if row.get("source_id")
    }
    responses = [row for row in load_json_records(response_path) if row.get("split") == "test"]

    def make_pair(row: dict):
        source = sources.get(row.get("source_id", ""))
        response = text_value(row.get("response"))
        labels = row.get("labels", [])
        if not source or not response:
            return None
        knowledge = truncate_text(text_value(source.get("source_info")))
        prompt = text_value(source.get("prompt"))
        if not knowledge:
            return None
        text = f"Knowledge: {knowledge}\nPrompt: {prompt}\nAnswer: {response}"
        return (text, bool(labels), row.get("task_type", source.get("task_type", "ragtruth")))

    pairs = sample_balanced(responses, make_pair, n_per_class, seed)
    n = len(pairs) // 2

    print(f"\n{'=' * 60}\nOFFICIAL BENCHMARK: RAGTruth\n{'=' * 60}")
    print(f"  Datasets: {source_path} + {response_path}")
    print(f"  Running on {len(pairs)} samples ({n} grounded + {n} hallucinated)...")
    print("  Mode: Grounded + S41 Entity Novelty (KAC + illusion + entity novelty)")
    results = evaluate_pairs_combined(pairs, workers, combined_verdict_fn=ragtruth_combined_verdict)
    return package_result(results, "RAGTruth official", f"{source_path} + {response_path}", n)


def benchmark_faithbench(downloads_dir: Path, n_per_class: int, seed: int, workers: int) -> dict:
    path = downloads_dir / "faithbench" / "FaithBench.csv"
    with open(path, newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))

    def make_pair(row: dict):
        source = text_value(row.get("source"))
        summary = text_value(row.get("summary"))
        worst_label = text_value(row.get("worst-label"))
        if not (source and summary and worst_label):
            return None
        has_issue = worst_label.lower() != "consistent"
        return (f"Knowledge: {truncate_text(source)}\nAnswer: {summary}", has_issue, worst_label)

    pairs = sample_balanced(rows, make_pair, n_per_class, seed)
    n = len(pairs) // 2

    print(f"\n{'=' * 60}\nOFFICIAL BENCHMARK: FaithBench\n{'=' * 60}")
    print(f"  Dataset: {path}")
    print(f"  Running on {len(pairs)} samples ({n} consistent + {n} flagged)...")
    print("  Mode: Grounded + S41 Entity Novelty (KAC + illusion + entity novelty)")
    results = evaluate_pairs_combined(
        pairs, workers, combined_verdict_fn=faithbench_combined_verdict
    )
    return package_result(results, "FaithBench official", str(path), n)
