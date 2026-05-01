"""HaluEval QA and Dialogue benchmarks."""

import random
from pathlib import Path

from .core import load_json_records, text_value, truncate_text
from .metrics import evaluate_pairs_combined, package_result
from .verdicts import halueval_dialogue_combined_verdict, halueval_qa_combined_verdict


def benchmark_halueval_qa(downloads_dir: Path, n_per_class: int, seed: int, workers: int) -> dict:
    path = downloads_dir / "halueval" / "qa_data.json"
    rows = load_json_records(path)

    def make_pair(row: dict):
        knowledge = text_value(row.get("knowledge"))
        question = text_value(row.get("question"))
        right = text_value(row.get("right_answer"))
        hallucinated = text_value(row.get("hallucinated_answer"))
        if not (knowledge and question and right and hallucinated):
            return None
        return [
            (
                f"Knowledge: {truncate_text(knowledge)}\nQuestion: {question}\nAnswer: {right}",
                False,
                "correct",
            ),
            (
                f"Knowledge: {truncate_text(knowledge)}\nQuestion: {question}\nAnswer: {hallucinated}",
                True,
                "hallucination",
            ),
        ]

    flat_pairs = []
    for row in rows:
        pair_bundle = make_pair(row)
        if pair_bundle:
            flat_pairs.extend(pair_bundle)
    positives = [pair for pair in flat_pairs if pair[1]]
    negatives = [pair for pair in flat_pairs if not pair[1]]
    rng = random.Random(seed)
    rng.shuffle(positives)
    rng.shuffle(negatives)
    n = min(n_per_class, len(positives), len(negatives))
    pairs = positives[:n] + negatives[:n]

    print(f"\n{'=' * 60}\nOFFICIAL BENCHMARK: HaluEval QA\n{'=' * 60}")
    print(f"  Dataset: {path}")
    print(f"  Running on {len(pairs)} samples ({n} correct + {n} hallucinated)...")
    print("  Mode: KAC/illusion + S39b grounding novelty recall-boost (threshold=0.25, TRIZ P25)")
    results = evaluate_pairs_combined(pairs, workers, halueval_qa_combined_verdict)
    return package_result(results, "HaluEval QA official", str(path), n)


def benchmark_halueval_dialogue(
    downloads_dir: Path, n_per_class: int, seed: int, workers: int
) -> dict:
    path = downloads_dir / "halueval" / "dialogue_data.json"
    rows = load_json_records(path)

    def make_pair(row: dict):
        knowledge = text_value(row.get("knowledge"))
        history = text_value(row.get("dialogue_history"))
        right = text_value(row.get("right_response"))
        hallucinated = text_value(row.get("hallucinated_response"))
        if not (knowledge and right and hallucinated):
            return None
        prompt = (
            f"Knowledge: {truncate_text(knowledge)}\nDialogue: {history}"
            if history
            else f"Knowledge: {truncate_text(knowledge)}"
        )
        return [
            (f"{prompt}\nResponse: {right}", False, "correct"),
            (f"{prompt}\nResponse: {hallucinated}", True, "hallucination"),
        ]

    flat_pairs = []
    for row in rows:
        pair_bundle = make_pair(row)
        if pair_bundle:
            flat_pairs.extend(pair_bundle)
    positives = [pair for pair in flat_pairs if pair[1]]
    negatives = [pair for pair in flat_pairs if not pair[1]]
    rng = random.Random(seed)
    rng.shuffle(positives)
    rng.shuffle(negatives)
    n = min(n_per_class, len(positives), len(negatives))
    pairs = positives[:n] + negatives[:n]

    print(f"\n{'=' * 60}\nOFFICIAL BENCHMARK: HaluEval Dialogue\n{'=' * 60}")
    print(f"  Dataset: {path}")
    print(f"  Running on {len(pairs)} samples ({n} correct + {n} hallucinated)...")
    print("  Mode: Grounded + S42 Entity Novelty (KAC + illusion + entity novelty)")
    results = evaluate_pairs_combined(
        pairs, workers, combined_verdict_fn=halueval_dialogue_combined_verdict
    )
    return package_result(results, "HaluEval Dialogue official", str(path), n)
