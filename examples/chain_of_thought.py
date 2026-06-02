#!/usr/bin/env python3
"""Use Case 2: Chain-of-Thought Verification — verify multi-step reasoning.

Demonstrates how to verify an ordered sequence of reasoning steps for
internal consistency (each step on its own) and contextual consistency
(each step against the accumulated context).

Run:
    python examples/chain_of_thought.py
"""

import sys

sys.path.insert(0, ".")

from pureason.reasoning import verify_chain
from pureason.reasoning.models import EpistemicChainReport


def example_valid_chain():
    """A correct chain — all steps should pass."""
    problem = "A store has 50 apples. A customer buys 12. How many remain?"
    steps = [
        "The store starts with 50 apples.",
        "A customer buys 12 apples.",
        "Remaining = 50 - 12 = 38.",
        "Therefore, the answer is 38.",
    ]

    report: EpistemicChainReport = verify_chain(problem, steps)

    print("=== Valid Chain ===")
    _print_report(report)
    print()
    return report


def example_arithmetic_error_chain():
    """A chain with an arithmetic error — step should be flagged."""
    problem = "What is the total of 15 and 27?"
    steps = [
        "We need to add the two numbers.",
        "15 + 27 = 43.",
        "Therefore, the answer is 43.",
    ]

    # Note: 15 + 27 = 42, so step 1 has an arithmetic error
    # (The answer step also carries the wrong value.)

    report = verify_chain(problem, steps)

    print("=== Arithmetic Error Chain ===")
    _print_report(report)
    print()
    return report


def example_contradiction_chain():
    """A chain where a later step contradicts an earlier one."""
    problem = "Describe the weather."
    steps = [
        "The temperature is 35 degrees Celsius.",
        "It is a very hot day.",
        "The roads are covered in ice due to freezing temperatures.",
    ]

    report = verify_chain(problem, steps)

    print("=== Contradiction Chain ===")
    _print_report(report)
    print()
    return report


def example_empty_chain():
    """Edge case: empty step list."""
    report = verify_chain("Any problem?", [])

    print("=== Empty Chain ===")
    print(f"  is_valid:         {report.is_valid}")
    print(f"  chain_confidence: {report.chain_confidence}")
    print(f"  summary:          {report.summary}")
    print()
    return report


def example_single_step():
    """Edge case: single-step chain (the step is both reasoning and answer)."""
    report = verify_chain(
        "What is 2 + 2?",
        ["2 + 2 = 4, so the answer is 4."],
    )

    print("=== Single-Step Chain ===")
    _print_report(report)
    print()
    return report


def _print_report(report: EpistemicChainReport):
    """Pretty-print an EpistemicChainReport."""
    print(f"  Problem:          {report.problem}")
    print(f"  is_valid:         {report.is_valid}")
    print(f"  chain_confidence: {report.chain_confidence:.4f}")
    print(f"  answer:           {report.answer}")
    print(f"  invalid_steps:    {report.invalid_steps}")
    print(f"  summary:          {report.summary}")
    for sv in report.steps:
        status = "OK" if sv.is_internally_valid and sv.is_contextually_valid else "FAIL"
        flags_str = ", ".join(sv.flags) if sv.flags else "none"
        print(
            f"    Step {sv.step_index}: [{status:>4s}] ECS={sv.ecs:3d} "
            f"int={sv.is_internally_valid} ctx={sv.is_contextually_valid} "
            f"flags=[{flags_str}]"
        )
        if sv.contradiction_with_step is not None:
            print(f"           contradicts step {sv.contradiction_with_step}")


if __name__ == "__main__":
    example_valid_chain()
    example_arithmetic_error_chain()
    example_contradiction_chain()
    example_empty_chain()
    example_single_step()
