"""HalluMix benchmark (2025) — task-agnostic multi-domain hallucination benchmark."""

import random
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from .core import BenchmarkExecutionError, run_pipeline_heuristic, text_value, truncate_text
from .metrics import package_result
from .semantic import _batch_semantic_scores, _get_st_model
from .verdicts import verdict_is_issue_grounded


def benchmark_hallumix(downloads_dir: Path, n_per_class: int, seed: int, workers: int) -> dict:
    """Evaluate on HalluMix — a task-agnostic, multi-domain hallucination benchmark.

    Format: Parquet with columns [question, answer, documents, hallucination_label].
    hallucination_label=True → hallucinated (positive class).

    Verdict: grounded (evidence-based signals only — has_illusions + has_contradictions).
    The context (documents list) is prepended as evidence, matching the RAGTruth approach.
    """
    path = downloads_dir / "hallumix" / "train.parquet"
    try:
        import pyarrow.parquet as pq  # type: ignore[import]
    except ImportError:
        raise SystemExit(
            "HalluMix benchmark requires pyarrow. Install with: pip3 install pyarrow"
        ) from None

    table = pq.read_table(path)
    rows = table.to_pydict()
    n_rows = table.num_rows

    def make_pair(idx: int):
        question = text_value(rows["question"][idx])
        answer = text_value(rows["answer"][idx])
        docs = rows["documents"][idx] or []
        label = bool(rows["hallucination_label"][idx])

        context_parts = [text_value(d) for d in docs[:5]]
        context = " ".join(p[:400] for p in context_parts if p)
        if context:
            text = (
                f"Context: {truncate_text(context, 3000)}\n\nQuestion: {question}\nAnswer: {answer}"
            )
        else:
            text = f"Question: {question}\nAnswer: {answer}"
        return (truncate_text(text), label, rows.get("source", ["hallumix"])[idx] or "hallumix")

    positives = [i for i in range(n_rows) if rows["hallucination_label"][i]]
    negatives = [i for i in range(n_rows) if not rows["hallucination_label"][i]]

    rng = random.Random(seed)
    pos_sample = rng.sample(positives, min(n_per_class, len(positives)))
    neg_sample = rng.sample(negatives, min(n_per_class, len(negatives)))
    indices = pos_sample + neg_sample
    rng.shuffle(indices)

    pairs = []
    for idx in indices:
        result = make_pair(idx)
        if result:
            pairs.append(result)

    n = len(pos_sample)
    print(f"\n{'=' * 60}\nOFFICIAL BENCHMARK: HalluMix (2025)\n{'=' * 60}")
    print(f"  Dataset: {path}  ({n_rows} total samples, {len(positives)} hallucinated)")
    print(f"  Running on {len(pairs)} samples ({n} hallucinated + {len(neg_sample)} faithful)...")

    st_model = _get_st_model()
    if st_model is not None:
        print("  Mode: Semantic cosine (all-MiniLM-L6-v2, threshold=0.99) + grounded heuristic")
        print(f"  Pre-computing semantic scores for {len(pairs)} pairs...")
        semantic_flags = _batch_semantic_scores(pairs, st_model, threshold=0.99)
    else:
        print(
            "  Mode: Grounded (evidence-based signals only — sentence-transformers not available)"
        )
        print("        Install with: pip3 install sentence-transformers")
        semantic_flags = {}

    def run_pair(pair) -> tuple[bool, bool]:
        text, is_issue, _ = pair
        try:
            verdict = run_pipeline_heuristic(text)
            predicted = verdict_is_issue_grounded(verdict) or semantic_flags.get(text[:100], False)
            return is_issue, predicted
        except BenchmarkExecutionError:
            return is_issue, semantic_flags.get(text[:100], False)

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
    return package_result(eval_results, "HalluMix official", str(path), n)
