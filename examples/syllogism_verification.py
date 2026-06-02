#!/usr/bin/env python3
"""Use Case 4: Syllogism Verification — formal logic checking.

Demonstrates PureReason's ability to verify logical arguments using
a multi-strategy approach:
  1. TF-IDF + Logistic Regression classifier (fast, data-driven)
  2. Z3 formal entailment (symbolic logic)
  3. Informal fallacy heuristics (hasty generalisation, circular reasoning)
  4. KAC consistency check (semantic overlap fallback)

Run:
    python examples/syllogism_verification.py
"""

import sys

sys.path.insert(0, ".")

from pureason.reasoning import verify_syllogism
from pureason.reasoning.models import EpistemicChainReport


def example_valid_syllogism():
    """Classic valid syllogism — conclusion follows from premises."""
    premises = [
        "All mammals are warm-blooded.",
        "Whales are mammals.",
    ]
    conclusion = "Whales are warm-blooded."

    report: EpistemicChainReport = verify_syllogism(premises, conclusion)

    print("=== Valid Syllogism ===")
    _print_report(premises, conclusion, report)
    print()
    return report


def example_invalid_syllogism():
    """Invalid syllogism — conclusion does not follow."""
    premises = [
        "All dogs are animals.",
        "All cats are animals.",
    ]
    conclusion = "Therefore, all dogs are cats."

    report = verify_syllogism(premises, conclusion)

    print("=== Invalid Syllogism ===")
    _print_report(premises, conclusion, report)
    print()
    return report


def example_hasty_generalisation():
    """Informal fallacy: specific instances → universal conclusion."""
    premises = [
        "John is tall.",
        "Mary is tall.",
    ]
    conclusion = "All people are tall."

    report = verify_syllogism(premises, conclusion)

    print("=== Hasty Generalisation ===")
    _print_report(premises, conclusion, report)
    print()
    return report


def example_modus_ponens():
    """Valid argument form: If P then Q; P; therefore Q."""
    premises = [
        "If it rains, the ground gets wet.",
        "It is raining.",
    ]
    conclusion = "The ground is wet."

    report = verify_syllogism(premises, conclusion)

    print("=== Modus Ponens ===")
    _print_report(premises, conclusion, report)
    print()
    return report


def example_three_premise_chain():
    """Transitive chain: A→B, B→C, therefore A→C."""
    premises = [
        "All birds have feathers.",
        "All animals with feathers can fly.",
        "Penguins are birds.",
    ]
    conclusion = "Penguins can fly."

    report = verify_syllogism(premises, conclusion)

    print("=== Three-Premise Chain (tricky — penguins can't fly) ===")
    _print_report(premises, conclusion, report)
    print()
    return report


def _print_report(premises, conclusion, report: EpistemicChainReport):
    """Pretty-print a syllogism verification report."""
    for i, p in enumerate(premises):
        print(f"  Premise {i + 1}: {p}")
    print(f"  Conclusion: {conclusion}")
    print("  ---")
    print(f"  is_valid:         {report.is_valid}")
    print(f"  chain_confidence: {report.chain_confidence:.2f}")
    print(f"  summary:          {report.summary}")
    for sv in report.steps:
        flags_str = ", ".join(sv.flags) if sv.flags else "none"
        print(
            f"    Step {sv.step_index}: ECS={sv.ecs:3d} "
            f"ctx_valid={sv.is_contextually_valid} "
            f"flags=[{flags_str}]"
        )


if __name__ == "__main__":
    example_valid_syllogism()
    example_invalid_syllogism()
    example_hasty_generalisation()
    example_modus_ponens()
    example_three_premise_chain()
