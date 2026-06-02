#!/usr/bin/env python3
"""Use Case 6: Arithmetic Repair — fix computation errors in text.

Demonstrates the deterministic arithmetic repair pipeline that finds
'A op B = C' patterns in text and corrects wrong results.

This is PureReason's core advantage over raw LLMs: formal arithmetic
verification + repair. LLM arithmetic mistakes become opportunities
for formal correction.

Run:
    python examples/arithmetic_repair.py
"""

import sys

sys.path.insert(0, ".")

from pureason.reasoning.repair import (
    _extract_letter_answer,
    _extract_numeric_answer,
    _majority_vote,
    _majority_vote_letters,
    _repair_arithmetic_in_step,
)


def example_repair():
    """Repair arithmetic errors in text."""
    cases = [
        # (input_text, should_repair)
        ("We computed 3 + 4 = 7 apples.", False),
        ("We computed 3 + 4 = 8 apples.", True),
        ("The product is 6 * 7 = 41.", True),
        ("Half of 10 is 10 / 2 = 5.", False),
        ("The difference is 100 - 37 = 64.", True),
    ]

    print("=== Arithmetic Repair ===")
    for text, expect_repair in cases:
        result = _repair_arithmetic_in_step(text)
        was_repaired = "[repaired]" in result
        status = "OK" if was_repaired == expect_repair else "UNEXPECTED"
        print(f"  {status:>10s}: {text}")
        if was_repaired:
            print(f"             → {result}")
    print()


def example_extract_numeric():
    """Extract the final numeric answer from text."""
    texts = [
        ("The answer is 42.", 42.0),
        ("Therefore, 3.14 is the result.", 3.14),
        ("No number here at all.", None),
        ("After calculation we get 100 items total.", 100.0),
    ]

    print("=== Extract Numeric Answer ===")
    for text, expected in texts:
        result = _extract_numeric_answer(text)
        # Allow None comparison and close-enough floats
        if result is None and expected is None:
            match = True
        elif result is not None and expected is not None:
            match = abs(result - expected) < 0.01
        else:
            match = False
        status = "OK" if match else "MISMATCH"
        print(f"  {status:>8s}: {text!r} → {result} (expected {expected})")
    print()


def example_extract_letter():
    """Extract MCQ letter answers from text."""
    texts = [
        ("Therefore the answer is A.", "A"),
        ("After analysis, the best answer is **B**.", "B"),
        ("ANSWER: C", "C"),
        ("No clear MCQ answer here.", None),
    ]

    print("=== Extract Letter Answer ===")
    for text, expected in texts:
        result = _extract_letter_answer(text)
        status = "OK" if result == expected else "MISMATCH"
        print(f"  {status:>8s}: {text!r} → {result!r} (expected {expected!r})")
    print()


def example_majority_vote():
    """Majority vote for aggregating multiple answers."""
    print("=== Majority Vote ===")

    # Numeric
    nums = [42.0, 42.0, 41.0, 42.0]
    print(f"  Numeric: {nums} → {_majority_vote(nums)}")

    nums_empty: list = []
    print(f"  Empty:   {nums_empty} → {_majority_vote(nums_empty)}")

    # Letters
    letters = ["A", "B", "A", "A", "C"]
    print(f"  Letters: {letters} → {_majority_vote_letters(letters)}")

    letters_with_none = [None, "B", None, "B"]
    print(f"  With None: {letters_with_none} → {_majority_vote_letters(letters_with_none)}")
    print()


if __name__ == "__main__":
    example_repair()
    example_extract_numeric()
    example_extract_letter()
    example_majority_vote()
