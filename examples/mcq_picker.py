#!/usr/bin/env python3
"""Use Case 5: Multiple-Choice Question Picker.

Demonstrates PureReason's MCQ answer selection by verifying each choice
against the question context and selecting the one with the highest ECS.

Run:
    python examples/mcq_picker.py
"""

import sys

sys.path.insert(0, ".")

from pureason.reasoning import pick_best_answer
from pureason.reasoning.mcq import AmbiguousAnswerError


def example_clear_winner():
    """One choice is clearly more defensible than the others."""
    question = "What is the capital of France?"
    choices = [
        "Berlin",
        "Paris",
        "Madrid",
        "Rome",
    ]

    best_idx, report = pick_best_answer(question, choices)

    print("=== Clear Winner ===")
    print(f"  Question: {question}")
    for i, c in enumerate(choices):
        marker = " ← best" if i == best_idx else ""
        print(f"    [{i}] {c}{marker}")
    print(f"  Selected index: {best_idx} ({choices[best_idx]})")
    print(f"  is_valid: {report.is_valid}")
    print(f"  chain_confidence: {report.chain_confidence:.4f}")
    print()
    return best_idx, report


def example_with_context():
    """Provide background context to improve discrimination."""
    question = "Based on the passage, which animal is the fastest?"
    choices = [
        "The cheetah can reach 70 mph.",
        "The lion can reach 50 mph.",
        "The elephant can reach 25 mph.",
    ]
    context = "African wildlife includes cheetahs, lions, and elephants."

    best_idx, report = pick_best_answer(question, choices, context=context)

    print("=== With Context ===")
    print(f"  Context:  {context}")
    print(f"  Question: {question}")
    for i, c in enumerate(choices):
        marker = " ← best" if i == best_idx else ""
        print(f"    [{i}] {c}{marker}")
    print(f"  Selected index: {best_idx}")
    print()
    return best_idx, report


def example_ambiguous_strict():
    """When choices are equally defensible, strict mode raises an error."""
    question = "Pick a color."
    choices = ["Red", "Blue"]

    print("=== Ambiguous (strict mode) ===")
    print(f"  Question: {question}")
    try:
        pick_best_answer(question, choices, strict=True)
        print("  No ambiguity detected.")
    except AmbiguousAnswerError as e:
        print(f"  AmbiguousAnswerError: {e}")
        print(f"  Tied indices: {e.tied_indices}, ECS: {e.ecs}")
    print()


def example_ambiguous_lenient():
    """In default (lenient) mode, ties are resolved to the first index
    and flagged with MCQ_AMBIGUOUS_ECS_TIE."""
    question = "Pick a color."
    choices = ["Red", "Blue"]

    best_idx, report = pick_best_answer(question, choices, strict=False)

    print("=== Ambiguous (lenient mode) ===")
    print(f"  Selected index: {best_idx}")
    if report.steps:
        flags = report.steps[0].flags
        print(f"  Flags: {flags}")
    print()
    return best_idx, report


if __name__ == "__main__":
    example_clear_winner()
    example_with_context()
    example_ambiguous_strict()
    example_ambiguous_lenient()
