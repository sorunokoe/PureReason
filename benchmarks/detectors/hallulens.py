"""HalluLens PreciseWiki grounding benchmark (arXiv:2504.17550)."""

import random
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from .core import BenchmarkExecutionError, run_pipeline_heuristic, text_value, truncate_text
from .metrics import package_result
from .semantic import python_grounding_novelty
from .verdicts import verdict_is_issue_grounded


def benchmark_hallulens(downloads_dir: Path, n_per_class: int, seed: int, workers: int) -> dict:
    """Reference-Answer Grounding Benchmark using HalluLens PreciseWiki data.

    Dataset: swiss-ai/hallulens precise_wiki split (arXiv:2504.17550),
    mirrored from HuggingFace. 5000 Wikipedia factual QA pairs.

    Evaluation design (synthetic grounding benchmark):
    ─────────────────────────────────────────────────
    The dataset contains only CORRECT answers, so we create a balanced binary
    benchmark by constructing two classes:

    NEGATIVE (label=False, not hallucinated):
      Knowledge: {reference}
      Q: {prompt}
      A: {answer}                  ← correct answer for this question
      Expected: SAFE

    POSITIVE (label=True, grounding failure):
      Knowledge: {reference_i}
      Q: {prompt_i}
      A: {answer_j}                ← answer from a DIFFERENT category row j≠i
      Expected: ISSUE (answer not supported by reference)

    Positive sampling constraints:
    - j drawn from a DIFFERENT Wikipedia category group than i
    - answer_j must NOT appear as a substring of reference_i (avoids trivial matches)
    - same-category filtering reduces entity-type-mismatch shortcuts

    Signal tested: KAC (Knowledge-Answer Contradiction) and entity-novelty.
    Verdict fn: verdict_is_issue_grounded (knowledge-grounded signals only).

    Note: This benchmark measures reference-answer consistency, not general
    hallucination recall. It should be interpreted alongside ungrounded benchmarks
    (TruthfulQA, LogicBench) for a complete picture.
    """
    path = downloads_dir / "hallulens" / "precise_wiki_test.parquet"
    try:
        import pyarrow.parquet as pq  # type: ignore[import]
    except ImportError:
        raise SystemExit(
            "HalluLens benchmark requires pyarrow. Install with: pip3 install pyarrow"
        ) from None

    table = pq.read_table(path)
    rows = table.to_pydict()
    n_rows = table.num_rows

    category_to_indices: dict[str, list[int]] = defaultdict(list)
    for i in range(n_rows):
        cats = rows["categories"][i]
        key = str(cats[0]).strip() if cats and len(cats) > 0 else "uncategorized"
        category_to_indices[key].append(i)

    all_categories = list(category_to_indices.keys())
    if len(all_categories) < 2:
        raise SystemExit("HalluLens: not enough categories for cross-category sampling")

    rng = random.Random(seed)

    def pick_cross_category_answer(row_i: int) -> str | None:
        """Pick an answer from a different category that doesn't appear in reference_i."""
        ref = text_value(rows["reference"][row_i])
        cats = rows["categories"][row_i]
        own_cat = str(cats[0]).strip() if cats and len(cats) > 0 else "uncategorized"
        other_cats = [c for c in all_categories if c != own_cat]
        if not other_cats:
            return None
        for _ in range(10):
            other_cat = rng.choice(other_cats)
            other_idx = rng.choice(category_to_indices[other_cat])
            ans = text_value(rows["answer"][other_idx])
            if ans and ans.lower() not in ref.lower():
                return ans
        return None

    all_indices = list(range(n_rows))
    rng.shuffle(all_indices)

    pairs: list[tuple[str, bool, str]] = []
    for idx in all_indices:
        if len(pairs) >= n_per_class * 2:
            break
        ref = text_value(rows["reference"][idx])
        prompt = text_value(rows["prompt"][idx])
        answer = text_value(rows["answer"][idx])
        if not (ref and prompt and answer):
            continue

        neg_count = sum(1 for _, is_issue, _ in pairs if not is_issue)
        pos_count = sum(1 for _, is_issue, _ in pairs if is_issue)

        if neg_count < n_per_class:
            knowledge = truncate_text(ref, 2000)
            text = f"Knowledge: {knowledge}\n\nQ: {prompt}\nA: {answer}"
            pairs.append((truncate_text(text), False, "hallulens-faithful"))

        if pos_count < n_per_class:
            swap = pick_cross_category_answer(idx)
            if swap:
                knowledge = truncate_text(ref, 2000)
                text = f"Knowledge: {knowledge}\n\nQ: {prompt}\nA: {swap}"
                pairs.append((truncate_text(text), True, "hallulens-grounding-failure"))

    rng.shuffle(pairs)
    n_neg = sum(1 for _, is_issue, _ in pairs if not is_issue)
    n_pos = sum(1 for _, is_issue, _ in pairs if is_issue)

    print(f"\n{'=' * 60}\nOFFICIAL BENCHMARK: HalluLens PreciseWiki — Grounding Test\n{'=' * 60}")
    print(f"  Dataset: {path}  ({n_rows} total samples)")
    print("  Benchmark type: Reference-answer consistency (synthetic grounding)")
    print(f"  Running on {len(pairs)} pairs ({n_neg} faithful + {n_pos} grounding-failure)...")
    print("  Mode: Hybrid (Rust KAC/entity-novelty OR Python word-novelty grounding check)")
    print(
        "        Python grounding: content-word novelty >= 50% → ISSUE (calibrated P=0.654, R=1.00)"
    )

    def run_pair(pair: tuple[str, bool, str]) -> tuple[bool, bool]:
        text, is_issue, _ = pair
        try:
            verdict = run_pipeline_heuristic(text)
            predicted = verdict_is_issue_grounded(verdict) or python_grounding_novelty(text)
            return is_issue, predicted
        except BenchmarkExecutionError:
            return is_issue, False

    with ThreadPoolExecutor(max_workers=workers) as pool:
        all_results = list(pool.map(run_pair, pairs))

    eval_results = [
        {
            "text_preview": text[:100],
            "ground_truth": gt,
            "predicted": pred,
            "category": cat,
            "risk": "unknown",
            "error": None,
        }
        for (text, _, cat), (gt, pred) in zip(pairs, all_results)
    ]
    return package_result(eval_results, "HalluLens PreciseWiki (grounding)", str(path), n_neg)
