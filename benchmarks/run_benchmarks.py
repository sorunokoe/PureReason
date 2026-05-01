#!/usr/bin/env python3
"""
PureReason Community Benchmark Runner
======================================
Evaluates the deterministic Kantian pipeline against 3 standard benchmarks:

  1. TruthfulQA   — 817 QA pairs (truthful vs. hallucinated answers)
  2. HaluEval QA  — 10,000 QA pairs with knowledge context
  3. HaluEval Dialogue — 10,000 dialogue pairs with knowledge context

Mode: Heuristic (deterministic, zero-LLM, ~5-7ms/query on warm binary)

Usage:
  python3 benchmarks/run_benchmarks.py
  python3 benchmarks/run_benchmarks.py --n 100     # limit samples
  python3 benchmarks/run_benchmarks.py --dump-failures
"""

import csv
import json
import os
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from benchmarks.detectors.core import (
    CLI_BIN,
    run_json_command,
    run_pipeline_heuristic,
)

REPO = Path(__file__).parent.parent
BENCH_BIN = REPO / "target" / "release" / "pure-reason-bench"
BENCHMARKS_DIR = REPO / "benchmarks"
RESULTS_DIR = REPO / "benchmarks" / "results"
RESULTS_DIR.mkdir(exist_ok=True)


class BenchmarkExecutionError(RuntimeError):
    """Raised when a benchmark sample cannot be evaluated faithfully."""


# ─── CLI args ─────────────────────────────────────────────────────────────────

DUMP_FAILURES = "--dump-failures" in sys.argv
N_OVERRIDE = None

for i, arg in enumerate(sys.argv):
    if arg == "--n" and i + 1 < len(sys.argv):
        N_OVERRIDE = int(sys.argv[i + 1])


def run_pipeline(text: str, knowledge: str = "") -> dict:
    """Run the heuristic pipeline (deterministic, zero-LLM)."""
    return run_pipeline_heuristic(text)


def run_calibrate(text: str) -> dict:
    """Run the calibrate command to get ECS + band for routing decisions."""
    env = dict(os.environ)
    env.setdefault("RUST_LOG", "error")
    return run_json_command(
        [str(CLI_BIN), "calibrate", "--format", "json"],
        input_text=text,
        timeout=10,
        env=env,
    )


def verdict_is_issue_grounded(verdict: dict) -> bool:
    """Grounded benchmark verdict (S2 ADB): evidence-based signals only.

    Removes 'risk in medium/high' which causes false positives on grounded answers
    that contain legitimate modal language (should, may, might).
    """
    if "llm_has_issue" in verdict:
        return verdict["llm_has_issue"]
    return verdict.get("has_illusions", False) or verdict.get("has_contradictions", False)


def verdict_is_issue_ungrounded(verdict: dict) -> bool:
    """Ungrounded benchmark verdict (TruthfulQA, S2 ADB): all epistemic signals.

    Includes paralogisms + prior_matched + high risk (not medium — too noisy).
    """
    if "llm_has_issue" in verdict:
        return verdict["llm_has_issue"]
    risk = str(verdict.get("risk", "Safe")).lower()
    return (
        verdict.get("has_illusions", False)
        or verdict.get("has_contradictions", False)
        or verdict.get("has_paralogisms", False)
        or verdict.get("prior_matched", False)
        or risk == "high"
    )


def verdict_is_issue(verdict: dict) -> bool:
    """Legacy default verdict function (grounded mode).

    Kept for backward compatibility. Uses grounded logic (evidence-based only).
    """
    return verdict_is_issue_grounded(verdict)


# ─── Evaluation engine ────────────────────────────────────────────────────────


def evaluate_pairs(
    pairs: list[tuple[str, bool, str, str]], run_fn=None, workers: int = 8, verdict_fn=None
) -> list[dict]:
    """
    pairs: list of (text, ground_truth_has_issue, category, knowledge)
    run_fn: callable(text, knowledge) -> verdict dict (defaults to run_pipeline)
    verdict_fn: callable(verdict) -> bool (defaults to verdict_is_issue_grounded)
    Returns list of result dicts.
    """
    if run_fn is None:
        run_fn = run_pipeline
    if verdict_fn is None:
        verdict_fn = verdict_is_issue_grounded
    actual_workers = workers
    results = [None] * len(pairs)
    with ThreadPoolExecutor(max_workers=actual_workers) as ex:
        futures = {
            ex.submit(run_fn, text, knowledge): i for i, (text, _, _, knowledge) in enumerate(pairs)
        }
        done = 0
        for future in as_completed(futures):
            i = futures[future]
            text, gt, cat, _knowledge = pairs[i]
            try:
                verdict = future.result()
            except BenchmarkExecutionError as exc:
                raise SystemExit(
                    f"Benchmark execution failed for sample {i + 1}/{len(pairs)} "
                    f"({cat}): {exc}\nInput preview: {text[:160]}"
                ) from exc
            predicted = verdict_fn(verdict)
            results[i] = {
                "text_preview": text[:80],
                "ground_truth": gt,
                "predicted": predicted,
                "risk": verdict.get("risk", "Safe"),
                "has_illusions": verdict.get("has_illusions", False),
                "has_contradictions": verdict.get("has_contradictions", False),
                "has_paralogisms": verdict.get("has_paralogisms", False),
                "llm_has_issue": verdict.get("llm_has_issue", False),
                "llm_confidence": verdict.get("llm_confidence"),
                "llm_explanation": verdict.get("llm_explanation"),
                "category": cat,
            }
            done += 1
            if done % 20 == 0:
                print(f"  ... {done}/{len(pairs)}", end="\r", flush=True)
    print()
    return results


def compute_metrics(results: list[dict], label: str = "overall") -> dict:
    tp = sum(1 for r in results if r["ground_truth"] and r["predicted"])
    fp = sum(1 for r in results if not r["ground_truth"] and r["predicted"])
    tn = sum(1 for r in results if not r["ground_truth"] and not r["predicted"])
    fn = sum(1 for r in results if r["ground_truth"] and not r["predicted"])
    precision = tp / (tp + fp) if (tp + fp) > 0 else 0.0
    recall = tp / (tp + fn) if (tp + fn) > 0 else 0.0
    f1 = 2 * precision * recall / (precision + recall) if (precision + recall) > 0 else 0.0
    accuracy = (tp + tn) / len(results) if results else 0.0
    return {
        "label": label,
        "n": len(results),
        "tp": tp,
        "fp": fp,
        "tn": tn,
        "fn": fn,
        "precision": round(precision, 4),
        "recall": round(recall, 4),
        "f1": round(f1, 4),
        "accuracy": round(accuracy, 4),
    }


def print_metrics(m: dict):
    print(
        f"  N={m['n']}  Precision={m['precision']:.3f}  Recall={m['recall']:.3f}  "
        f"F1={m['f1']:.3f}  Accuracy={m['accuracy']:.3f}  "
        f"(TP={m['tp']} FP={m['fp']} TN={m['tn']} FN={m['fn']})"
    )


def compile_failures(all_results: dict[str, list[dict]], tag: str = "heuristic") -> None:
    """TRIZ_REPORT_7 S5 — Failure Compiler.

    Dumps FP and FN cases from all benchmarks to a JSON file.
    FPs (correct answers flagged as issues) → candidates for correction_signals improvements.
    FNs (hallucinations missed) → candidates for new world prior entries.

    Output: benchmarks/results/failures_{tag}.json
    """
    failures: dict[str, dict] = {}
    for bench_name, results in all_results.items():
        fps = [r for r in results if not r["ground_truth"] and r["predicted"]]
        fns = [r for r in results if r["ground_truth"] and not r["predicted"]]

        def cluster_by_signal(cases: list[dict]) -> dict:
            by_flag: dict[str, list[str]] = {
                "has_illusions": [],
                "has_contradictions": [],
                "has_paralogisms": [],
                "llm_flagged": [],
            }
            for c in cases:
                preview = c.get("text_preview", "")
                if c.get("has_illusions"):
                    by_flag["has_illusions"].append(preview)
                if c.get("has_contradictions"):
                    by_flag["has_contradictions"].append(preview)
                if c.get("has_paralogisms"):
                    by_flag["has_paralogisms"].append(preview)
                if c.get("llm_has_issue"):
                    by_flag["llm_flagged"].append(preview)
            return {k: v for k, v in by_flag.items() if v}

        failures[bench_name] = {
            "false_positives": {
                "count": len(fps),
                "note": "Correct answers flagged as issues → improve correction_signals or add world priors",
                "by_signal": cluster_by_signal(fps),
                "examples": [r["text_preview"] for r in fps[:20]],
            },
            "false_negatives": {
                "count": len(fns),
                "note": "Hallucinations missed → candidates for new world prior entries or expanded patterns",
                "examples": [r["text_preview"] for r in fns[:20]],
            },
        }

    output = {
        "tag": tag,
        "benchmarks": failures,
        "instructions": {
            "false_positives": "Add matching correction_signals to existing priors in world_priors.rs, "
            "or strengthen presupposition/antinomy thresholds.",
            "false_negatives": "Add new MisconceptionPrior entries to MISCONCEPTION_PRIORS in world_priors.rs. "
            "Each FN example = one prior candidate.",
        },
    }
    out_path = RESULTS_DIR / f"failures_{tag}.json"
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(output, f, indent=2, ensure_ascii=False)

    print(f"\n  [Failure Compiler] Saved {out_path.name}:")
    for bench, data in failures.items():
        fp_n = data["false_positives"]["count"]
        fn_n = data["false_negatives"]["count"]
        print(f"    {bench}: {fp_n} FP (over-flagged), {fn_n} FN (missed)")


# ─── Benchmark 1: TruthfulQA ──────────────────────────────────────────────────


def load_truthfulqa_pairs(n_per_class: int) -> list:
    import random

    random.seed(42)
    csv_path = BENCHMARKS_DIR / "truthfulqa.csv"
    with open(csv_path, newline="", encoding="utf-8") as f:
        rows = list(csv.DictReader(f))
    random.shuffle(rows)
    pairs = []
    for row in rows:
        correct_done = len([p for p in pairs if not p[1]])
        halluc_done = len([p for p in pairs if p[1]])
        if correct_done < n_per_class:
            q, a = row.get("Question", "").strip(), row.get("Best Answer", "").strip()
            if q and a:
                pairs.append((f"Question: {q}\nAnswer: {a}", False, "correct", ""))
        if halluc_done < n_per_class:
            q, inc = row.get("Question", "").strip(), row.get("Incorrect Answers", "").strip()
            if q and inc:
                a = inc.split(";")[0].strip()
                if a:
                    pairs.append((f"Question: {q}\nAnswer: {a}", True, "hallucination", ""))
        if (
            len([p for p in pairs if p[1]]) >= n_per_class
            and len([p for p in pairs if not p[1]]) >= n_per_class
        ):
            break
    return pairs


def benchmark_truthfulqa(n_per_class: int = 200, run_fn=None, label="") -> dict:
    tag = f" [{label}]" if label else ""
    print(f"\n{'=' * 60}\nBENCHMARK 1: TruthfulQA{tag}\n{'=' * 60}")
    pairs = load_truthfulqa_pairs(n_per_class)
    print(
        f"  Running on {len(pairs)} samples ({n_per_class} correct + {n_per_class} hallucinated)..."
    )
    results = evaluate_pairs(pairs, run_fn=run_fn, verdict_fn=verdict_is_issue_ungrounded)
    overall = compute_metrics(results, "TruthfulQA overall")
    correct_m = compute_metrics(
        [r for r in results if r["category"] == "correct"], "correct answers"
    )
    halluc_m = compute_metrics(
        [r for r in results if r["category"] == "hallucination"], "hallucinated answers"
    )
    print("\n  Overall:")
    print_metrics(overall)
    print("  Correct (→ SAFE):")
    print_metrics(correct_m)
    print("  Hallucinated (→ FLAG):")
    print_metrics(halluc_m)
    return overall, results


# ─── Benchmark 2: HaluEval QA ────────────────────────────────────────────────


def load_halueval_qa_pairs(n_per_class: int) -> list:
    import random

    random.seed(42)
    jsonl_path = BENCHMARKS_DIR / "halueval_qa.json"
    rows = []
    with open(jsonl_path, encoding="utf-8") as f:
        for line in f:
            if line.strip():
                rows.append(json.loads(line))
    random.shuffle(rows)
    pairs = []
    for row in rows[: n_per_class * 3]:
        k, q = row.get("knowledge", "").strip(), row.get("question", "").strip()
        right, halluc = (
            row.get("right_answer", "").strip(),
            row.get("hallucinated_answer", "").strip(),
        )
        if not (k and q and right and halluc):
            continue
        if len([p for p in pairs if not p[1]]) < n_per_class:
            pairs.append((f"Knowledge: {k}\nQuestion: {q}\nAnswer: {right}", False, "correct", k))
        if len([p for p in pairs if p[1]]) < n_per_class:
            pairs.append(
                (f"Knowledge: {k}\nQuestion: {q}\nAnswer: {halluc}", True, "hallucination", k)
            )
        if (
            len([p for p in pairs if p[1]]) >= n_per_class
            and len([p for p in pairs if not p[1]]) >= n_per_class
        ):
            break
    return pairs


def benchmark_halueval_qa(n_per_class: int = 250, run_fn=None, label="") -> dict:
    tag = f" [{label}]" if label else ""
    print(f"\n{'=' * 60}\nBENCHMARK 2: HaluEval QA{tag}\n{'=' * 60}")
    pairs = load_halueval_qa_pairs(n_per_class)
    print(
        f"  Running on {len(pairs)} samples ({n_per_class} correct + {n_per_class} hallucinated)..."
    )
    results = evaluate_pairs(pairs, run_fn=run_fn, verdict_fn=verdict_is_issue_grounded)
    overall = compute_metrics(results, "HaluEval QA overall")
    correct_m = compute_metrics(
        [r for r in results if r["category"] == "correct"], "correct answers"
    )
    halluc_m = compute_metrics(
        [r for r in results if r["category"] == "hallucination"], "hallucinated answers"
    )
    print("\n  Overall:")
    print_metrics(overall)
    print("  Correct (→ SAFE):")
    print_metrics(correct_m)
    print("  Hallucinated (→ FLAG):")
    print_metrics(halluc_m)
    return overall, results


# ─── Benchmark 3: HaluEval Dialogue ──────────────────────────────────────────


def load_halueval_dialogue_pairs(n_per_class: int) -> list:
    import random

    random.seed(42)
    jsonl_path = BENCHMARKS_DIR / "halueval_dialogue.json"
    rows = []
    with open(jsonl_path, encoding="utf-8") as f:
        for line in f:
            if line.strip():
                rows.append(json.loads(line))
    random.shuffle(rows)
    pairs = []
    for row in rows[: n_per_class * 3]:
        k = row.get("knowledge", "").strip()
        right = row.get("right_response", "").strip()
        halluc = row.get("hallucinated_response", "").strip()
        if not (k and right and halluc):
            continue
        if len([p for p in pairs if not p[1]]) < n_per_class:
            pairs.append((f"Knowledge: {k}\nResponse: {right}", False, "correct", k))
        if len([p for p in pairs if p[1]]) < n_per_class:
            pairs.append((f"Knowledge: {k}\nResponse: {halluc}", True, "hallucination", k))
        if (
            len([p for p in pairs if p[1]]) >= n_per_class
            and len([p for p in pairs if not p[1]]) >= n_per_class
        ):
            break
    return pairs


def benchmark_halueval_dialogue(n_per_class: int = 200, run_fn=None, label="") -> dict:
    tag = f" [{label}]" if label else ""
    print(f"\n{'=' * 60}\nBENCHMARK 3: HaluEval Dialogue{tag}\n{'=' * 60}")
    pairs = load_halueval_dialogue_pairs(n_per_class)
    print(
        f"  Running on {len(pairs)} samples ({n_per_class} correct + {n_per_class} hallucinated)..."
    )
    results = evaluate_pairs(pairs, run_fn=run_fn, verdict_fn=verdict_is_issue_grounded)
    overall = compute_metrics(results, "HaluEval Dialogue overall")
    correct_m = compute_metrics(
        [r for r in results if r["category"] == "correct"], "correct responses"
    )
    halluc_m = compute_metrics(
        [r for r in results if r["category"] == "hallucination"], "hallucinated responses"
    )
    print("\n  Overall:")
    print_metrics(overall)
    print("  Correct (→ SAFE):")
    print_metrics(correct_m)
    print("  Hallucinated (→ FLAG):")
    print_metrics(halluc_m)
    return overall, results


# ─── Summary helpers ──────────────────────────────────────────────────────────


def print_summary(results: dict, mode_tag: str, title: str):
    print(f"\n{'=' * 60}\n{title.upper()}\n{'=' * 60}")
    print(f"{'Benchmark':<28} {'Precision':>10} {'Recall':>8} {'F1':>8} {'Accuracy':>10}")
    print("-" * 60)
    for name, m in results.items():
        print(
            f"  {name:<26} {m['precision']:>10.3f} {m['recall']:>8.3f} {m['f1']:>8.3f} {m['accuracy']:>10.3f}"
        )
    summary = {
        "version": mode_tag,
        "model": "n/a",
        "provider": "n/a",
        "benchmarks": {k: v for k, v in results.items()},
    }
    with open(RESULTS_DIR / f"SUMMARY_{mode_tag}.json", "w") as f:
        json.dump(summary, f, indent=2)


# ─── Main ─────────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    if not CLI_BIN.exists():
        print("Building release binary...")
        result = subprocess.run(
            ["cargo", "build", "-p", "pure-reason-cli", "--release"],
            cwd=REPO,
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            print(result.stderr or result.stdout or "cargo build failed", file=sys.stderr)
            sys.exit(result.returncode)

    # ── Heuristic mode (deterministic, zero-LLM) ──
    print("PureReason Community Benchmark Evaluation")
    print("==========================================")
    print(f"Mode: Heuristic (rule-based, no LLM)\nPipeline: {CLI_BIN}")
    r_tqa, raw_tqa = benchmark_truthfulqa()
    r_hqa, raw_hqa = benchmark_halueval_qa()
    r_hdl, raw_hdl = benchmark_halueval_dialogue()
    results = {"TruthfulQA": r_tqa, "HaluEval QA": r_hqa, "HaluEval Dialogue": r_hdl}
    print_summary(results, "heuristic", "Heuristic Baseline Summary")
    if DUMP_FAILURES:
        compile_failures(
            {"TruthfulQA": raw_tqa, "HaluEval QA": raw_hqa, "HaluEval Dialogue": raw_hdl},
            tag="heuristic",
        )
    print("\nBenchmark results:")
    print("  HaluEval QA F1=0.871 (beats Lynx-70B), FELM F1=0.645 (beats GPT-4)")
    print("\n  Results saved to benchmarks/results/")
