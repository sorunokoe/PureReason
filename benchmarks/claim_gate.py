#!/usr/bin/env python3
"""S102: Honest Claim Gate — validate benchmark claims before publishing.

Compares two JSON result files (baseline vs method) and asserts that the
performance difference is statistically significant and honestly reported.

Usage
-----
    # Run fast harness first:
    python3 benchmarks/fast_harness.py --out results/run_a.json

    # Validate a claim:
    python3 benchmarks/claim_gate.py baseline=results/run_a.json method=results/run_b.json

    # Or compare two methods within the SAME harness run:
    python3 benchmarks/claim_gate.py --file results/run_a.json --baseline baseline --method falsify

Exit codes
----------
  0  Claim valid: significant improvement, all integrity checks pass.
  1  Claim rejected: see output for reason.
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Paired McNemar test (duplicated from fast_harness to be standalone)
# ---------------------------------------------------------------------------


def _mcnemar_p(b: int, c: int) -> float:
    total = b + c
    if total == 0:
        return 1.0
    if total < 25:
        lo = min(b, c)
        p = 0.0
        for k in range(lo + 1):
            p += math.comb(total, k) * (0.5**total)
        return min(1.0, 2 * p)
    chi2 = (abs(b - c) - 1) ** 2 / total
    x = chi2 / 2.0
    t = 1.0 / (1.0 + 0.3275911 * math.sqrt(x))
    poly = t * (
        0.254829592 + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429)))
    )
    erfc_approx = poly * math.exp(-x)
    return min(1.0, erfc_approx)


def _wilson_ci(correct: int, n: int, z: float = 1.96) -> tuple[float, float]:
    if n == 0:
        return 0.0, 0.0
    p = correct / n
    denom = 1 + z * z / n
    centre = (p + z * z / (2 * n)) / denom
    margin = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / denom
    return max(0.0, centre - margin), min(1.0, centre + margin)


# ---------------------------------------------------------------------------
# Integrity checks
# ---------------------------------------------------------------------------


def _check(condition: bool, message: str) -> bool:
    if not condition:
        print(f"  FAIL: {message}", flush=True)
    return condition


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main() -> int:
    import argparse

    parser = argparse.ArgumentParser(
        description="S102 Honest Claim Gate",
        formatter_class=argparse.RawTextHelpFormatter,
    )
    parser.add_argument(
        "--file",
        default=None,
        help="Single harness JSON file containing results for multiple methods.",
    )
    parser.add_argument(
        "--baseline",
        default="baseline",
        help="Baseline method name (default: 'baseline').",
    )
    parser.add_argument(
        "--method",
        default=None,
        help="Method to validate against baseline.",
    )
    parser.add_argument(
        "--baseline-file",
        default=None,
        help="Separate JSON file for baseline results.",
    )
    parser.add_argument(
        "--method-file",
        default=None,
        help="Separate JSON file for method results.",
    )
    parser.add_argument(
        "--min-n",
        type=int,
        default=40,
        help="Minimum n per method for claim to be valid (default 40).",
    )
    parser.add_argument(
        "--alpha",
        type=float,
        default=0.05,
        help="Significance level (default 0.05).",
    )
    args = parser.parse_args()

    print("\n" + "=" * 60, flush=True)
    print("  S102 Honest Claim Gate", flush=True)
    print("=" * 60 + "\n", flush=True)

    passed = True

    # --- Load results ---
    if args.file:
        path = Path(args.file)
        if not path.exists():
            print(f"ERROR: File not found: {path}", file=sys.stderr)
            return 1
        data = json.loads(path.read_text())
        all_results: list[dict] = data.get("results", [])

        if not args.method:
            available = sorted({r["method"] for r in all_results})
            print(
                f"  Available methods in file: {available}\n"
                "  Use --method to specify which to validate.",
                flush=True,
            )
            return 1

        baseline_res = {r["id"]: r for r in all_results if r["method"] == args.baseline}
        method_res = {r["id"]: r for r in all_results if r["method"] == args.method}
    elif args.baseline_file and args.method_file:
        b_data = json.loads(Path(args.baseline_file).read_text())
        m_data = json.loads(Path(args.method_file).read_text())
        b_results = b_data.get("results", [])
        m_results = m_data.get("results", [])
        baseline_res = {r["id"]: r for r in b_results}
        method_res = {r["id"]: r for r in m_results}
        args.method = m_data.get("method", "method")
    else:
        print(
            "ERROR: Provide --file or (--baseline-file + --method-file).",
            file=sys.stderr,
        )
        return 1

    # --- Check 1: same question set ---
    common_ids = set(baseline_res) & set(method_res)
    only_in_baseline = set(baseline_res) - set(method_res)
    only_in_method = set(method_res) - set(baseline_res)

    print(f"  Baseline ({args.baseline}): {len(baseline_res)} items", flush=True)
    print(f"  Method   ({args.method}):   {len(method_res)} items", flush=True)
    print(f"  Common   : {len(common_ids)} items", flush=True)

    passed &= _check(
        len(only_in_baseline) == 0 and len(only_in_method) == 0,
        f"Question sets differ: {len(only_in_baseline)} only in baseline, "
        f"{len(only_in_method)} only in method. "
        "Claim requires IDENTICAL question sets for paired comparison.",
    )

    # --- Check 2: minimum n ---
    passed &= _check(
        len(common_ids) >= args.min_n,
        f"n={len(common_ids)} < min_n={args.min_n}. "
        f"Need at least {args.min_n} items for reliable comparison.",
    )

    # --- Check 3: label consistency ---
    label_mismatch = sum(
        1 for i in common_ids if baseline_res[i]["label"] != method_res[i]["label"]
    )
    passed &= _check(
        label_mismatch == 0,
        f"{label_mismatch} items have different ground-truth labels. "
        "Files must use the same dataset split.",
    )

    # --- Accuracy stats ---
    n = len(common_ids)
    b_correct = sum(baseline_res[i]["correct"] for i in common_ids)
    m_correct = sum(method_res[i]["correct"] for i in common_ids)
    b_acc = b_correct / n if n else 0
    m_acc = m_correct / n if n else 0
    b_lo, b_hi = _wilson_ci(b_correct, n)
    m_lo, m_hi = _wilson_ci(m_correct, n)

    b = sum(
        1 for i in common_ids if method_res[i]["correct"] == 1 and baseline_res[i]["correct"] == 0
    )
    c = sum(
        1 for i in common_ids if baseline_res[i]["correct"] == 1 and method_res[i]["correct"] == 0
    )
    p_val = _mcnemar_p(b, c)
    delta = m_acc - b_acc

    print(f"\n  Baseline : {b_acc:.1%} [CI95: {b_lo:.1%}–{b_hi:.1%}]", flush=True)
    print(f"  Method   : {m_acc:.1%} [CI95: {m_lo:.1%}–{m_hi:.1%}]", flush=True)
    print(f"  Delta    : {delta:+.1%}", flush=True)
    print(f"  McNemar  : b={b} c={c}  p={p_val:.4f} (α={args.alpha})", flush=True)

    # --- Check 4: positive delta ---
    passed &= _check(
        delta > 0,
        f"Method did not improve over baseline (Δ={delta:+.1%}). Cannot claim improvement.",
    )

    # --- Check 5: statistical significance ---
    passed &= _check(
        p_val < args.alpha,
        f"Difference is NOT statistically significant (p={p_val:.4f} >= α={args.alpha}). "
        "Need more data or a larger effect size before publishing this claim.",
    )

    # --- Check 6: CI does not overlap (soft check, warning only) ---
    ci_overlap = b_hi > m_lo  # baseline CI extends above method CI lower bound
    if ci_overlap:
        print(
            "  WARN: CI ranges overlap. The improvement may not be robust at this n. "
            "Consider n ≥ 160 for ≤7pp CI.",
            flush=True,
        )

    # --- Verdict ---
    print("\n" + "=" * 60, flush=True)
    if passed:
        print("  ✓ CLAIM VALID", flush=True)
        print(
            f"  {args.method} ({m_acc:.1%}) significantly outperforms "
            f"{args.baseline} ({b_acc:.1%}) "
            f"by {delta:+.1%} (McNemar p={p_val:.4f}).",
            flush=True,
        )
    else:
        print("  ✗ CLAIM REJECTED — see FAIL messages above", flush=True)
    print("=" * 60 + "\n", flush=True)

    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
