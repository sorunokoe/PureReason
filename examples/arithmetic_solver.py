#!/usr/bin/env python3
"""Use Case 3: Arithmetic Word Problem Solver.

Demonstrates PureReason's ability to:
  - Extract numbers from natural language (digits and words)
  - Detect the intended operation (+, -, *, /)
  - Compute the answer with a verified reasoning chain
  - Handle multi-step and ratio/proportion problems

Run:
    python examples/arithmetic_solver.py
"""

import sys

sys.path.insert(0, ".")

from pureason.reasoning import solve_arithmetic
from pureason.reasoning.arithmetic import _detect_operation, _extract_numbers, _safe_eval


def example_safe_eval():
    """Demonstrate the safe expression evaluator (no exec/eval)."""
    expressions = [
        ("2 + 3", 5.0),
        ("10 - 4", 6.0),
        ("6 * 7", 42.0),
        ("10 / 4", 2.5),
        ("2 ** 10", 1024.0),
        ("(3 + 4) * 2", 14.0),
        ("-5 + 3", -2.0),
        ("5 / 0", None),           # division by zero → None
        ("import os", None),       # not arithmetic → None
    ]

    print("=== Safe Expression Evaluator ===")
    for expr, expected in expressions:
        result = _safe_eval(expr)
        status = "OK" if result == expected else "MISMATCH"
        print(f"  {status:>8s}: _safe_eval({expr!r}) = {result}  (expected {expected})")
    print()


def example_number_extraction():
    """Extract numbers from natural language text."""
    texts = [
        "There are 3 apples and 10 bananas.",
        "The price is 3.14 dollars.",
        "Temperature is -5 degrees.",
        "No numeric content here.",
        "The factory produced 1,000 units.",
    ]

    print("=== Number Extraction ===")
    for text in texts:
        nums = _extract_numbers(text)
        print(f"  {text}")
        print(f"    → {nums}")
    print()


def example_operation_detection():
    """Detect the intended arithmetic operation from problem text."""
    problems = [
        ("How many total items if we add 3 more?", "+"),
        ("How many are left after removing 5?", "-"),
        ("What is the average speed?", "/"),
        ("A car travels 60 mph for 4 hours. How far?", "*"),
        ("They split the 100 dollars equally.", "/"),
    ]

    print("=== Operation Detection ===")
    for text, expected_op in problems:
        detected = _detect_operation(text)
        status = "OK" if detected == expected_op else "MISMATCH"
        print(f"  {status:>8s}: {text}")
        print(f"            detected={detected!r}, expected={expected_op!r}")
    print()


def example_word_problems():
    """Solve complete arithmetic word problems end-to-end."""
    problems = [
        "Maria has 15 apples. She buys 8 more. How many apples does she have in total?",
        "A store has 120 items. They sold 45 items. How many are left?",
        "Each box contains 6 items. There are 9 boxes. How many items altogether?",
        "There are 48 cookies to share among 8 children. How many does each child get?",
    ]

    print("=== Word Problem Solver ===")
    for problem in problems:
        report = solve_arithmetic(problem)
        print(f"  Problem: {problem}")
        print(f"  Answer:  {report.answer}")
        print(f"  Valid:   {report.is_valid}")
        print(f"  Conf:    {report.chain_confidence:.4f}")
        for sv in report.steps:
            print(f"    Step {sv.step_index}: {sv.step_text[:70]}")
        print()


if __name__ == "__main__":
    example_safe_eval()
    example_number_extraction()
    example_operation_detection()
    example_word_problems()
