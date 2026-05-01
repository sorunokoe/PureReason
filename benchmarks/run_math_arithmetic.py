#!/usr/bin/env python3
"""
MATH Arithmetic Reasoning Benchmark — PureReason evaluation.

Tests PureReason's solve_arithmetic() on single-operation arithmetic word
problems. Measures whether the solver extracts the correct numerical answer
from natural-language problems.

Ground-truth problems are self-contained (no external dataset required).
Problems cover: addition, subtraction, multiplication, division,
rate/speed, unit conversion, and multi-step phrasing.

Usage:
    python3 benchmarks/run_math_arithmetic.py [--seed 42]

Random baseline: not applicable (open-ended numeric answer, not MC).
Metric: answer_match = |predicted - ground_truth| < tolerance.
"""

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

# ---------------------------------------------------------------------------
# Curated problem set (50 problems, verified correct)
# ---------------------------------------------------------------------------

PROBLEMS: list[dict] = [
    # --- ADDITION ---
    {
        "problem": "A store has 34 apples and receives 58 more. How many apples are there in total?",
        "answer": 92,
        "op": "+",
    },
    {
        "problem": "A train travels 250 miles on Monday and 310 miles on Tuesday. What is the total distance?",
        "answer": 560,
        "op": "+",
    },
    {
        "problem": "John has 127 stamps. He buys 89 more. How many stamps does he have now?",
        "answer": 216,
        "op": "+",
    },
    {
        "problem": "A factory produces 1450 units on day one and 1320 units on day two. What is the total production?",
        "answer": 2770,
        "op": "+",
    },
    {
        "problem": "A school has 348 boys and 412 girls. How many students are there in total?",
        "answer": 760,
        "op": "+",
    },
    {
        "problem": "Sarah earned $85 on Saturday and $63 on Sunday. How much did she earn in total?",
        "answer": 148,
        "op": "+",
    },
    {
        "problem": "A truck carries 750 kg of sand and 430 kg of gravel. What is the total weight?",
        "answer": 1180,
        "op": "+",
    },
    {
        "problem": "A library has 2340 fiction books and 1780 non-fiction books. How many books in total?",
        "answer": 4120,
        "op": "+",
    },
    {
        "problem": "Mike ran 5 miles on Monday and 8 miles on Wednesday. How many miles did he run total?",
        "answer": 13,
        "op": "+",
    },
    {
        "problem": "A basket holds 48 oranges and 37 apples. How many fruits are there in total?",
        "answer": 85,
        "op": "+",
    },
    # --- SUBTRACTION ---
    {
        "problem": "A shop had 200 items and sold 73. How many items are left?",
        "answer": 127,
        "op": "-",
    },
    {
        "problem": "A tank holds 500 liters and 175 liters were used. How much water remains?",
        "answer": 325,
        "op": "-",
    },
    {
        "problem": "Lisa had $340 and spent $128. How much does she have left?",
        "answer": 212,
        "op": "-",
    },
    {
        "problem": "A city had a population of 85000 and 3200 people moved away. What is the new population?",
        "answer": 81800,
        "op": "-",
    },
    {
        "problem": "A bag of flour weighed 25 kg. After using 8 kg, how much remains?",
        "answer": 17,
        "op": "-",
    },
    {
        "problem": "Tom scored 95 points and Bob scored 67 points. By how many points did Tom win?",
        "answer": 28,
        "op": "-",
    },
    {
        "problem": "A rope is 48 meters long. If 15 meters are cut off, how long is the remaining rope?",
        "answer": 33,
        "op": "-",
    },
    {
        "problem": "A farmer harvested 620 bushels and sold 285. How many bushels remain?",
        "answer": 335,
        "op": "-",
    },
    {
        "problem": "A container had 800 gallons and 350 gallons leaked out. How many gallons are left?",
        "answer": 450,
        "op": "-",
    },
    {
        "problem": "A book has 384 pages. Maria has read 147 pages. How many pages remain?",
        "answer": 237,
        "op": "-",
    },
    # --- MULTIPLICATION ---
    {
        "problem": "A car travels 60 miles per hour for 4 hours. How far does it travel?",
        "answer": 240,
        "op": "*",
    },
    {
        "problem": "A factory produces 150 units per day. How many units does it produce in 6 days?",
        "answer": 900,
        "op": "*",
    },
    {
        "problem": "A box contains 24 chocolates. How many chocolates are in 7 boxes?",
        "answer": 168,
        "op": "*",
    },
    {
        "problem": "A worker earns $15 per hour and works 8 hours a day. How much does the worker earn in a day?",
        "answer": 120,
        "op": "*",
    },
    {
        "problem": "A rectangle is 12 meters wide and 9 meters long. What is its area?",
        "answer": 108,
        "op": "*",
    },
    {
        "problem": "A jar holds 32 cookies. How many cookies are in 5 jars?",
        "answer": 160,
        "op": "*",
    },
    {
        "problem": "A plane flies 500 miles per hour for 3 hours. What is the total distance traveled?",
        "answer": 1500,
        "op": "*",
    },
    {
        "problem": "A package weighs 4 pounds. What is the total weight of 12 packages?",
        "answer": 48,
        "op": "*",
    },
    {
        "problem": "There are 365 days in a year. How many days are in 3 years?",
        "answer": 1095,
        "op": "*",
    },
    {
        "problem": "A store sells 45 items per week. How many items does it sell in 8 weeks?",
        "answer": 360,
        "op": "*",
    },
    # --- DIVISION ---
    {
        "problem": "A train travels 120 miles in 2 hours. What is its speed in miles per hour?",
        "answer": 60,
        "op": "/",
    },
    {
        "problem": "A factory produced 900 units in 6 days. How many units per day on average?",
        "answer": 150,
        "op": "/",
    },
    {
        "problem": "250 apples are packed equally into 5 boxes. How many apples are in each box?",
        "answer": 50,
        "op": "/",
    },
    {
        "problem": "A car uses 48 gallons of fuel on a 384-mile trip. How many miles per gallon?",
        "answer": 8,
        "op": "/",
    },
    {
        "problem": "A rope of 72 meters is cut into 9 equal pieces. How long is each piece?",
        "answer": 8,
        "op": "/",
    },
    {
        "problem": "A class of 35 students is divided into groups of 7. How many groups are there?",
        "answer": 5,
        "op": "/",
    },
    {
        "problem": "A total of $480 is shared equally among 8 people. How much does each person get?",
        "answer": 60,
        "op": "/",
    },
    {
        "problem": "A bag of 144 candies is divided equally into 12 bags. How many candies per bag?",
        "answer": 12,
        "op": "/",
    },
    {
        "problem": "A runner completes a 26-mile marathon in 2 hours. What is the average speed in miles per hour?",
        "answer": 13,
        "op": "/",
    },
    {
        "problem": "A tank of 360 liters is filled from a pipe delivering 45 liters per minute. How many minutes to fill?",
        "answer": 8,
        "op": "/",
    },
    # --- MIXED / RATE ---
    {
        "problem": "A cyclist rides at 15 mph for 3 hours. How many miles does the cyclist cover?",
        "answer": 45,
        "op": "*",
    },
    {
        "problem": "If 6 workers can build a wall in 10 days, how many days would 60 workers take?",
        "answer": 1,
        "op": "/",
    },
    {
        "problem": "A pool holds 500 gallons. A pump removes 25 gallons per hour. How many hours to empty the pool?",
        "answer": 20,
        "op": "/",
    },
    {
        "problem": "There are 52 weeks in a year. How many weeks in 4 years?",
        "answer": 208,
        "op": "*",
    },
    {
        "problem": "A store sells 320 items in 8 days. What is the average number of items sold per day?",
        "answer": 40,
        "op": "/",
    },
    {
        "problem": "A farmer plants 5 seeds per row and has 45 rows. How many seeds in total?",
        "answer": 225,
        "op": "*",
    },
    {
        "problem": "A recipe needs 3 cups of flour for 12 cookies. How many cups are needed for 48 cookies?",
        "answer": 12,
        "op": "*",
    },
    {
        "problem": "A satellite orbits Earth every 90 minutes. How many orbits in 630 minutes?",
        "answer": 7,
        "op": "/",
    },
    {
        "problem": "A bus travels 240 km in 4 hours. What is the average speed in km/h?",
        "answer": 60,
        "op": "/",
    },
    {
        "problem": "A field has 18 rows of corn with 25 plants per row. How many corn plants are there in total?",
        "answer": 450,
        "op": "*",
    },
]


# ---------------------------------------------------------------------------
# Evaluation
# ---------------------------------------------------------------------------


def _answer_from_report(report) -> float | None:
    """Extract the predicted numeric answer from an EpistemicChainReport."""
    import re

    # Scan steps for "the answer is X" or "= X"
    if hasattr(report, "steps"):
        for step in reversed(report.steps):
            text = getattr(step, "step_text", str(step))
            for pattern in [
                r"answer is ([-+]?\d*\.?\d+)",
                r"= ([-+]?\d*\.?\d+)",
                r"Therefore.*?([-+]?\d*\.?\d+)",
                r"([-+]?\d*\.?\d+)\s*\.$",
            ]:
                m = re.search(pattern, text, re.IGNORECASE)
                if m:
                    try:
                        return float(m.group(1))
                    except ValueError:
                        pass
    return None


def _is_correct(predicted: float | None, expected: float, tol: float = 0.01) -> bool:
    if predicted is None:
        return False
    return abs(predicted - expected) <= tol * max(1.0, abs(expected))


def run_math_benchmark(seed: int = 42) -> dict:
    from pureason.reasoning import solve_arithmetic

    problems = PROBLEMS  # fixed 50-item set

    correct = 0
    parseable = 0
    results: list[dict] = []

    print("PureReason — MATH Arithmetic Reasoning Benchmark")
    print("=" * 60)
    print(f"n={len(problems)}  answer_tolerance=1%")
    print()
    print(f"{'#':>3}  {'OP':>2}  {'Expected':>10}  {'Predicted':>10}  {'✓?':>3}")
    print("-" * 45)

    for i, item in enumerate(problems):
        report = solve_arithmetic(item["problem"])
        pred = _answer_from_report(report)
        ok = _is_correct(pred, item["answer"])
        if pred is not None:
            parseable += 1
        if ok:
            correct += 1
        results.append(
            {
                "problem": item["problem"][:60],
                "expected": item["answer"],
                "predicted": pred,
                "correct": ok,
                "op": item["op"],
            }
        )
        pred_str = f"{pred:.4g}" if pred is not None else "N/A"
        print(
            f"{i + 1:>3}  {item['op']:>2}  {item['answer']:>10}  {pred_str:>10}  {'✓' if ok else '✗':>3}"
        )

    accuracy = correct / len(problems)
    parse_rate = parseable / len(problems)

    by_op: dict[str, dict] = {}
    for op in ["+", "-", "*", "/"]:
        op_items = [r for r in results if r["op"] == op]
        if op_items:
            op_acc = sum(1 for r in op_items if r["correct"]) / len(op_items)
            by_op[op] = {"n": len(op_items), "accuracy": round(op_acc, 4)}

    print()
    print("=" * 60)
    print(f"Overall accuracy:  {accuracy:.3f}  ({correct}/{len(problems)})")
    print(f"Parse rate:        {parse_rate:.3f}  ({parseable}/{len(problems)})")
    print()
    print("By operation:")
    for op, stats in by_op.items():
        names = {"+": "Addition", "-": "Subtraction", "*": "Multiplication", "/": "Division"}
        print(f"  {names[op]:>15}: {stats['accuracy']:.3f}  (n={stats['n']})")
    print("=" * 60)

    return {
        "benchmark": "MATH Arithmetic Reasoning",
        "n": len(problems),
        "seed": seed,
        "accuracy": round(accuracy, 4),
        "correct": correct,
        "parse_rate": round(parse_rate, 4),
        "by_operation": by_op,
        "per_item": results,
    }


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--out", type=str, default=None)
    args = parser.parse_args()

    results = run_math_benchmark(seed=args.seed)

    out_path = args.out or str(Path(__file__).parent / "results" / "math_arithmetic_results.json")
    Path(out_path).parent.mkdir(parents=True, exist_ok=True)
    with open(out_path, "w") as f:
        json.dump(results, f, indent=2)
    print(f"\nResults saved to {out_path}")


if __name__ == "__main__":
    main()
