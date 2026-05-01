#!/usr/bin/env python3
"""Reasoning Chain Verification Benchmark.

Tests PureReason's ability to verify the correctness of multi-step reasoning chains:
  - Arithmetic chain verification (S47 arithmetic oracle + vCoT)
  - Syllogism validation (structural consistency via KAC)

This is distinct from the hallucination-detection benchmarks.  Here we evaluate
whether PureReason can act as a *general reasoning verifier*: given a chain of
reasoning steps, correctly identify which chains contain errors.

Task definition (binary classification):
  - Positive class (label=True):  chain is VALID  (no errors, is_valid=True)
  - Negative class (label=False): chain is FLAWED (contains injected error)

Metrics reported: Precision, Recall, F1, Accuracy.
Random baseline for each sub-task: 0.500 (balanced classes).

Usage:
    python3 benchmarks/run_reasoning_verification.py [--n N] [--seed SEED]
"""

from __future__ import annotations

import argparse
import json
import random
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

# Add repo root to path
sys.path.insert(0, str(Path(__file__).parent.parent))
from pureason.reasoning import verify_chain, verify_syllogism

# ---------------------------------------------------------------------------
# Metrics helpers
# ---------------------------------------------------------------------------


@dataclass
class BenchResult:
    name: str
    tp: int
    tn: int
    fp: int
    fn: int

    @property
    def total(self) -> int:
        return self.tp + self.tn + self.fp + self.fn

    @property
    def precision(self) -> float:
        return self.tp / (self.tp + self.fp) if (self.tp + self.fp) else 0.0

    @property
    def recall(self) -> float:
        return self.tp / (self.tp + self.fn) if (self.tp + self.fn) else 0.0

    @property
    def f1(self) -> float:
        p, r = self.precision, self.recall
        return 2 * p * r / (p + r) if (p + r) else 0.0

    @property
    def accuracy(self) -> float:
        return (self.tp + self.tn) / self.total if self.total else 0.0

    def __str__(self) -> str:
        return (
            f"{self.name:<40} "
            f"P={self.precision:.3f}  R={self.recall:.3f}  "
            f"F1={self.f1:.3f}  Acc={self.accuracy:.3f}  "
            f"(n={self.total})"
        )


def _run_classification(
    name: str,
    problems: list[tuple[list[str], bool]],  # (steps, is_valid)
    predictor: Callable[[list[str]], bool],
) -> BenchResult:
    tp = tn = fp = fn = 0
    for steps, expected_valid in problems:
        predicted_valid = predictor(steps)
        if expected_valid and predicted_valid:
            tp += 1
        elif not expected_valid and not predicted_valid:
            tn += 1
        elif expected_valid and not predicted_valid:
            fn += 1
        else:
            fp += 1
    return BenchResult(name=name, tp=tp, tn=tn, fp=fp, fn=fn)


# ---------------------------------------------------------------------------
# Sub-benchmark 1: Arithmetic Chain Verification
# ---------------------------------------------------------------------------


def _generate_arithmetic_problems(n: int, rng: random.Random) -> list[tuple[list[str], bool]]:
    """Generate n valid + n flawed arithmetic reasoning chains."""
    ops = [
        ("+", lambda a, b: a + b),
        ("-", lambda a, b: a - b),
        ("*", lambda a, b: a * b),
        ("/", lambda a, b: int(a // b)),
    ]
    problems: list[tuple[list[str], bool]] = []

    for _ in range(n):
        op_sym, op_fn = rng.choice(ops)
        if op_sym == "/":
            b = rng.choice([2, 4, 5, 10])
            a = b * rng.randint(2, 20)
        else:
            a = rng.randint(2, 200)
            b = rng.randint(2, 50)
        result = op_fn(a, b)

        op_names = {"+": "sum", "-": "difference", "*": "product", "/": "quotient"}
        op_name = op_names[op_sym]

        # Correct chain
        correct_steps = [
            f"The given values are {a} and {b}.",
            f"We need to compute the {op_name}.",
            f"Computing: {a} {op_sym} {b} = {result}.",
            f"Therefore the answer is {result}.",
        ]
        problems.append((correct_steps, True))

        # Flawed chain: inject arithmetic error
        delta = rng.choice([-7, -5, -3, -2, 2, 3, 5, 7, 10, -10]) * rng.randint(1, 4)
        error_result = result + delta
        if error_result == result:
            error_result = result + 11
        flawed_steps = [
            f"The given values are {a} and {b}.",
            f"We need to compute the {op_name}.",
            f"Computing: {a} {op_sym} {b} = {error_result}.",  # WRONG
            f"Therefore the answer is {error_result}.",
        ]
        problems.append((flawed_steps, False))

    rng.shuffle(problems)
    return problems


# ---------------------------------------------------------------------------
# Sub-benchmark 2: Syllogism Validity Verification
# ---------------------------------------------------------------------------

_VALID_SYLLOGISMS: list[tuple[list[str], str]] = [
    # Modus ponens
    (["All mammals are warm-blooded.", "Dolphins are mammals."], "Dolphins are warm-blooded."),
    (["If it rains, the ground gets wet.", "It is raining."], "The ground is wet."),
    (
        ["All prime numbers greater than 2 are odd.", "17 is a prime number greater than 2."],
        "17 is odd.",
    ),
    (
        ["Every student who passes the exam gets a certificate.", "Alice passed the exam."],
        "Alice gets a certificate.",
    ),
    (["All metals conduct electricity.", "Copper is a metal."], "Copper conducts electricity."),
    # Modus tollens
    (["If it is sunny, Bob goes running.", "Bob is not going running."], "It is not sunny."),
    (
        ["All fish breathe through gills.", "The dolphin does not breathe through gills."],
        "The dolphin is not a fish.",
    ),
    # Hypothetical syllogism
    (["If A implies B.", "B implies C."], "A implies C."),
    (
        ["All squares are rectangles.", "All rectangles have four sides."],
        "All squares have four sides.",
    ),
    (
        ["If the temperature drops below 0, water freezes.", "If water freezes, pipes may burst."],
        "If the temperature drops below 0, pipes may burst.",
    ),
]

_INVALID_SYLLOGISMS: list[tuple[list[str], str]] = [
    # Affirming the consequent
    (["If it rains, the ground is wet.", "The ground is wet."], "It is raining."),
    (["All cats are animals.", "Rex is an animal."], "Rex is a cat."),
    # Denying the antecedent
    (["If it is sunny, Bob goes running.", "It is not sunny."], "Bob is not going running."),
    (["All birds have wings.", "Rex does not have wings."], "Rex is not a bird."),
    # Non sequitur
    (["The sun is a star.", "Stars produce light."], "All light comes from the sun."),
    # Undistributed middle
    (["All dogs are mammals.", "All cats are mammals."], "All dogs are cats."),
    # Equivocation
    (
        ["Nothing is better than eternal happiness.", "A sandwich is better than nothing."],
        "A sandwich is better than eternal happiness.",
    ),
    # Hasty generalisation (injected)
    (
        ["John is a doctor and is dishonest.", "Jane is a doctor and is dishonest."],
        "All doctors are dishonest.",
    ),
    # Wrong quantifier
    (["Some birds can fly.", "Penguins are birds."], "Penguins can fly."),
    # Circular
    (
        ["The Bible is true because it says so.", "The Bible says it is true."],
        "Therefore the Bible is true.",
    ),
]


def _generate_syllogism_problems(n: int, rng: random.Random) -> list[tuple[list[str], bool]]:
    """n valid + n invalid syllogism chains."""
    valid_pool = list(_VALID_SYLLOGISMS) * (n // len(_VALID_SYLLOGISMS) + 2)
    invalid_pool = list(_INVALID_SYLLOGISMS) * (n // len(_INVALID_SYLLOGISMS) + 2)
    rng.shuffle(valid_pool)
    rng.shuffle(invalid_pool)

    problems: list[tuple[list[str], bool]] = []
    for prems, concl in valid_pool[:n]:
        steps = [*list(prems), f"Therefore: {concl}"]
        problems.append((steps, True))
    for prems, concl in invalid_pool[:n]:
        steps = [*list(prems), f"Therefore: {concl}"]
        problems.append((steps, False))

    rng.shuffle(problems)
    return problems


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--n", type=int, default=50, help="Problems per class (default 50)")
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--output", help="Write JSON summary to this file")
    args = parser.parse_args()

    rng = random.Random(args.seed)

    print("PureReason — Reasoning Chain Verification Benchmark")
    print("=" * 60)
    print(f"n/class: {args.n}   seed: {args.seed}")
    print()

    results: list[BenchResult] = []

    # ----- Sub-benchmark 1: Arithmetic -----
    print(f"{'─' * 60}\nSUB-BENCHMARK 1: Arithmetic Chain Verification")
    print("  Verifying step-by-step arithmetic reasoning chains.")
    print(f"  VALID chains: {args.n}  |  FLAWED chains (wrong result): {args.n}")
    arith_problems = _generate_arithmetic_problems(args.n, rng)

    def arith_predict(steps: list[str]) -> bool:
        report = verify_chain("Arithmetic problem", steps)
        return report.is_valid

    arith_result = _run_classification(
        "Arithmetic Chain Verification", arith_problems, arith_predict
    )
    results.append(arith_result)
    print(f"  {arith_result}")
    print("  Random baseline: P=0.500 R=1.000 F1=0.667 Acc=0.500")

    # ----- Sub-benchmark 2: Syllogism -----
    print(f"\n{'─' * 60}\nSUB-BENCHMARK 2: Syllogism Validity Verification")
    print("  Distinguishing valid from invalid logical syllogisms.")
    print(f"  VALID: {args.n}  |  INVALID (logical fallacies): {args.n}")
    syllogism_problems = _generate_syllogism_problems(args.n, rng)

    def syllogism_predict(steps: list[str]) -> bool:
        # steps = [premise1, premise2, ..., "Therefore: conclusion"]
        # Route through the Z3 formal logic verifier
        conclusion = steps[-1]
        premises = steps[:-1]
        report = verify_syllogism(premises, conclusion)
        return report.is_valid

    syl_result = _run_classification(
        "Syllogism Validity Verification", syllogism_problems, syllogism_predict
    )
    results.append(syl_result)
    print(f"  {syl_result}")
    print("  Random baseline: P=0.500 R=1.000 F1=0.667 Acc=0.500")

    # ----- Summary -----
    print(f"\n{'=' * 60}\nSUMMARY")
    print(f"{'Benchmark':<40} {'Precision':>9} {'Recall':>7} {'F1':>7} {'Acc':>7}")
    print(f"{'─' * 60}")
    for r in results:
        print(f"{r.name:<40} {r.precision:>9.3f} {r.recall:>7.3f} {r.f1:>7.3f} {r.accuracy:>7.3f}")
    print(f"{'─' * 60}")
    print(f"{'Random baseline':<40} {'0.500':>9} {'1.000':>7} {'0.667':>7} {'0.500':>7}")

    if args.output:
        summary = {
            "n_per_class": args.n,
            "seed": args.seed,
            "results": [
                {
                    "name": r.name,
                    "precision": round(r.precision, 4),
                    "recall": round(r.recall, 4),
                    "f1": round(r.f1, 4),
                    "accuracy": round(r.accuracy, 4),
                    "tp": r.tp,
                    "tn": r.tn,
                    "fp": r.fp,
                    "fn": r.fn,
                }
                for r in results
            ],
        }
        Path(args.output).write_text(json.dumps(summary, indent=2))
        print(f"\nSummary written to {args.output}")


if __name__ == "__main__":
    main()
