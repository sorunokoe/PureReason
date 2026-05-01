"""Evaluation helpers: pair runners, metrics, confidence intervals, sampling."""

import random
from concurrent.futures import ThreadPoolExecutor, as_completed

from . import core as _core
from .core import BenchmarkExecutionError, run_pipeline_claims, run_pipeline_heuristic
from .verdicts import verdict_is_issue_grounded


def evaluate_pairs(
    pairs: list[tuple[str, bool, str]],
    workers: int,
    verdict_fn=None,
) -> list[dict]:
    """Evaluate pairs with an optional task-type-specific verdict function.

    Args:
        pairs: list of (text, ground_truth, category) tuples.
        workers: thread pool size.
        verdict_fn: callable(verdict: dict) -> bool. Defaults to verdict_is_issue_grounded.
    """
    if verdict_fn is None:
        verdict_fn = verdict_is_issue_grounded

    results: list[dict | None] = [None] * len(pairs)
    with ThreadPoolExecutor(max_workers=workers) as executor:
        futures = {
            executor.submit(run_pipeline_heuristic, text): index
            for index, (text, _, _) in enumerate(pairs)
        }
        done = 0
        for future in as_completed(futures):
            index = futures[future]
            text, ground_truth, category = pairs[index]
            try:
                verdict = future.result()
            except BenchmarkExecutionError as exc:
                raise SystemExit(
                    f"Benchmark execution failed for sample {index + 1}/{len(pairs)} "
                    f"({category}): {exc}\nInput preview: {text[:160]}"
                ) from exc
            results[index] = {
                "text_preview": text[:100],
                "ground_truth": ground_truth,
                "predicted": verdict_fn(verdict),
                "category": category,
                "risk": verdict.get("risk", "Safe"),
                "error": verdict.get("error"),
            }
            done += 1
            if done % 20 == 0:
                print(f"  ... {done}/{len(pairs)}", end="\r", flush=True)
    print()
    return [item for item in results if item is not None]


def evaluate_pairs_claims(pairs: list[tuple[str, bool, str]], workers: int) -> list[dict]:
    """Evaluate pairs using claims mode (OR rule: any risky claim = issue).

    Used for FELM where per-segment factual accuracy matters.
    The OR rule: if ANY claim in the answer has risk >= Low → issue.
    """
    results: list[dict | None] = [None] * len(pairs)
    with ThreadPoolExecutor(max_workers=workers) as executor:
        futures = {
            executor.submit(run_pipeline_claims, text): index
            for index, (text, _, _) in enumerate(pairs)
        }
        done = 0
        for future in as_completed(futures):
            index = futures[future]
            text, ground_truth, category = pairs[index]
            try:
                verdict = future.result()
            except BenchmarkExecutionError as exc:
                raise SystemExit(
                    f"Benchmark execution failed for sample {index + 1}/{len(pairs)} "
                    f"({category}): {exc}\nInput preview: {text[:160]}"
                ) from exc
            risk = str(verdict.get("overall_risk", verdict.get("risk", "Safe"))).lower()
            predicted = (
                verdict.get("risky_count", verdict.get("risky_claims", 0)) > 0
                or risk in ("medium", "high")
                or verdict.get("has_illusions", False)
                or verdict.get("has_contradictions", False)
            )
            results[index] = {
                "text_preview": text[:100],
                "ground_truth": ground_truth,
                "predicted": predicted,
                "category": category,
                "risk": verdict.get("overall_risk", verdict.get("risk", "Safe")),
                "error": verdict.get("error"),
            }
            done += 1
            if done % 20 == 0:
                print(f"  ... {done}/{len(pairs)}", end="\r", flush=True)
    print()
    return [item for item in results if item is not None]


def evaluate_pairs_combined(
    pairs: list[tuple[str, bool, str]],
    workers: int,
    combined_verdict_fn,
) -> list[dict]:
    """Evaluate pairs using a combined text+verdict predicate.

    Unlike evaluate_pairs (which only sees the verdict dict), this passes BOTH
    the original text AND the verdict to the predicate, enabling hybrid signals
    like arithmetic verification alongside heuristic analysis.
    Used by benchmark_felm for the S3 NPD arithmetic extension.
    """
    results: list[dict | None] = [None] * len(pairs)
    with ThreadPoolExecutor(max_workers=workers) as executor:
        futures = {
            executor.submit(run_pipeline_heuristic, text): index
            for index, (text, _, _) in enumerate(pairs)
        }
        done = 0
        for future in as_completed(futures):
            index = futures[future]
            text, ground_truth, category = pairs[index]
            try:
                verdict = future.result()
            except BenchmarkExecutionError as exc:
                raise SystemExit(
                    f"Benchmark execution failed for sample {index + 1}/{len(pairs)} "
                    f"({category}): {exc}\nInput preview: {text[:160]}"
                ) from exc
            results[index] = {
                "text_preview": text[:100],
                "ground_truth": ground_truth,
                "predicted": combined_verdict_fn(text, verdict),
                "category": category,
                "risk": verdict.get("risk", "Safe"),
                "error": verdict.get("error"),
            }
            done += 1
            if done % 20 == 0:
                print(f"  ... {done}/{len(pairs)}", end="\r", flush=True)
    print()
    return [item for item in results if item is not None]


def compute_metrics(results: list[dict], label: str) -> dict:
    tp = sum(1 for row in results if row["ground_truth"] and row["predicted"])
    fp = sum(1 for row in results if not row["ground_truth"] and row["predicted"])
    tn = sum(1 for row in results if not row["ground_truth"] and not row["predicted"])
    fn = sum(1 for row in results if row["ground_truth"] and not row["predicted"])
    precision = tp / (tp + fp) if (tp + fp) else 0.0
    recall = tp / (tp + fn) if (tp + fn) else 0.0
    f1 = 2 * precision * recall / (precision + recall) if (precision + recall) else 0.0
    accuracy = (tp + tn) / len(results) if results else 0.0
    errors = sum(1 for row in results if row.get("error"))
    n = len(results)
    return {
        "label": label,
        "n": n,
        "tp": tp,
        "fp": fp,
        "tn": tn,
        "fn": fn,
        "precision": round(precision, 4),
        "recall": round(recall, 4),
        "f1": round(f1, 4),
        "accuracy": round(accuracy, 4),
        "errors": errors,
        "ci_95": {
            "precision": wilson_ci_95(precision, tp + fp) if (tp + fp) else None,
            "recall": wilson_ci_95(recall, tp + fn) if (tp + fn) else None,
            "f1": se_ci_95(f1, n),
            "accuracy": wilson_ci_95(accuracy, n),
        },
    }


def wilson_ci_95(p: float, n: int) -> dict:
    """Wilson score 95% confidence interval for a proportion p with n observations.

    More accurate than the normal approximation for small n or extreme p.
    Returns {"lower": float, "upper": float, "margin": float}.
    """
    if n == 0:
        return {"lower": 0.0, "upper": 1.0, "margin": 0.5}
    z = 1.96
    z2 = z * z
    denominator = 1.0 + z2 / n
    center = (p + z2 / (2 * n)) / denominator
    half_width = z * ((p * (1 - p) / n + z2 / (4 * n * n)) ** 0.5) / denominator
    lower = max(0.0, round(center - half_width, 4))
    upper = min(1.0, round(center + half_width, 4))
    return {"lower": lower, "upper": upper, "margin": round(half_width, 4)}


def se_ci_95(f1: float, n: int) -> dict:
    """95% CI for F1 using standard error approximation (treats F1 as proportion).

    This is an approximation — the true distribution of F1 is complex.
    For large n (>50) this is sufficiently accurate for benchmark reporting.
    """
    if n == 0 or f1 <= 0.0 or f1 >= 1.0:
        return {"lower": round(f1, 4), "upper": round(f1, 4), "margin": 0.0}
    se = (f1 * (1 - f1) / n) ** 0.5
    margin = 1.96 * se
    return {
        "lower": max(0.0, round(f1 - margin, 4)),
        "upper": min(1.0, round(f1 + margin, 4)),
        "margin": round(margin, 4),
    }


def print_metrics(metrics: dict) -> None:
    ci = metrics.get("ci_95", {})
    f1_ci = ci.get("f1") or {}
    f1_lo = f1_ci.get("lower", metrics["f1"])
    f1_hi = f1_ci.get("upper", metrics["f1"])
    print(
        f"  N={metrics['n']}  Precision={metrics['precision']:.3f}  "
        f"Recall={metrics['recall']:.3f}  F1={metrics['f1']:.3f} "
        f"[95% CI {f1_lo:.3f}–{f1_hi:.3f}]  "
        f"Accuracy={metrics['accuracy']:.3f}  Errors={metrics['errors']}"
    )


def package_result(
    results: list[dict], label: str, dataset_path: str, sample_size_per_class: int
) -> dict:
    metrics = compute_metrics(results, label)
    print_metrics(metrics)
    error_examples = []
    for row in results:
        if row.get("error"):
            error_examples.append({"preview": row["text_preview"], "error": row["error"]})
            if len(error_examples) >= 3:
                break
    if error_examples:
        print(f"  Example error: {error_examples[0]['error']}")
    return {
        "metrics": metrics,
        "dataset_path": dataset_path,
        "sample_size_per_class": sample_size_per_class,
        "error_examples": error_examples,
    }


def sample_balanced(
    rows: list[dict],
    make_pair,
    n_per_class: int,
    seed: int,
    holdout: bool | None = None,
) -> list[tuple[str, bool, str]]:
    """Sample n_per_class balanced pairs from rows.

    holdout=True (S53): evaluates on a SECOND independent draw of n_per_class items.
    The first n_per_class items (seeds 0..n-1) represent the "calibration exposure"
    and are SKIPPED. Evaluation runs on items n..2n-1. This eliminates test-set
    leakage for benchmarks where thresholds were tuned on the seed=42 draw.
    If holdout is None, uses the module-level _HOLDOUT_MODE flag from core.
    """
    if holdout is None:
        holdout = _core._HOLDOUT_MODE

    positives = []
    negatives = []
    for row in rows:
        pair = make_pair(row)
        if pair is None:
            continue
        if pair[1]:
            positives.append(pair)
        else:
            negatives.append(pair)

    rng = random.Random(seed)
    rng.shuffle(positives)
    rng.shuffle(negatives)

    if holdout:
        n_avail = min(len(positives), len(negatives))
        skip = min(n_per_class, n_avail // 2)
        n = min(n_per_class, n_avail - skip)
        pos_slice = positives[skip : skip + n]
        neg_slice = negatives[skip : skip + n]
    else:
        n = min(n_per_class, len(positives), len(negatives))
        pos_slice = positives[:n]
        neg_slice = negatives[:n]

    return pos_slice + neg_slice
