#!/usr/bin/env python3
"""Use Case 1: ReasoningGuard — Verify any text with ECS scoring.

Demonstrates the primary PureReason entry point for verifying text.
The ReasoningGuard checks text using the Epistemic Confidence Score (ECS),
repairs arithmetic errors, and tracks quality degradation over time.

Run:
    python examples/guard_verification.py
"""

import sys

sys.path.insert(0, ".")

from pureason.guard import ReasoningGuard, VerificationResult


def example_basic_verification():
    """Verify text and inspect the VerificationResult fields."""
    guard = ReasoningGuard(threshold=60)

    text = "Water boils at 100 degrees Celsius at sea level."
    result: VerificationResult = guard.verify(text)

    print("=== Basic Verification ===")
    print(f"Input:      {text}")
    print(f"ECS:        {result.ecs}/100")
    print(f"Provenance: {result.provenance}")  # "verified", "repaired", or "flagged"
    print(f"Repaired:   {result.repaired}")
    print(f"Text out:   {result.text}")
    print()
    return result


def example_threshold_levels():
    """Show how different thresholds change the provenance outcome."""
    claims = [
        "The Earth orbits the Sun.",
        "2 + 2 = 5 so the total is wrong.",
        "The answer is both yes and no at the same time.",
    ]

    print("=== Threshold Comparison ===")
    for threshold in (40, 60, 80):
        guard = ReasoningGuard(threshold=threshold, repair=True)
        print(f"\n--- threshold={threshold} ---")
        for claim in claims:
            r = guard.verify(claim)
            print(f"  [{r.provenance:>8s}] ECS={r.ecs:5.1f}  {claim[:60]}")
    print()


def example_arithmetic_repair():
    """Demonstrate automatic arithmetic repair."""
    guard = ReasoningGuard(threshold=60, repair=True)

    texts = [
        "3 + 4 = 7 so the answer is correct.",  # correct — no repair
        "3 + 4 = 8 so the answer is correct.",  # wrong — repaired
        "6 * 7 = 41 which gives the total.",  # wrong — repaired
        "10 / 2 = 5 items per group.",  # correct — no repair
    ]

    print("=== Arithmetic Repair ===")
    for text in texts:
        r = guard.verify(text)
        if r.repaired:
            print(f"  REPAIRED: {r.original}")
            print(f"       =>   {r.text}")
        else:
            print(f"  OK:       {text}")
    print()


def example_degradation_tracking():
    """Show the degradation warning when quality drops over time."""
    import warnings

    from pureason.guard import ReasoningDegradationWarning, _ReputationTracker

    tracker = _ReputationTracker(window=3, baseline_window=6, drop=5.0)
    guard = ReasoningGuard(
        threshold=60,
        source_label="my_llm",
        warn_on_degradation=True,
        tracker=tracker,
    )

    # Simulate a sequence of ECS scores — first good, then degrading
    good_texts = ["The sky is blue."] * 6  # will get ~75 ECS each
    bad_texts = ["Maybe yes maybe no."] * 3  # will get lower ECS

    print("=== Degradation Tracking ===")
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always")
        for t in good_texts + bad_texts:
            guard.verify(t)

    degradation_warnings = [
        w for w in caught if issubclass(w.category, ReasoningDegradationWarning)
    ]
    if degradation_warnings:
        print(f"  Degradation detected: {degradation_warnings[0].message}")
    else:
        print("  No degradation detected (scores stayed stable).")
    print()


def example_decision_logic():
    """Show a complete agent decision workflow."""
    guard = ReasoningGuard(threshold=70)

    agent_outputs = [
        "Paris is the capital of France.",
        "The patient must have cancer based on a headache.",
        "2 + 3 = 6 so there are six items.",
    ]

    print("=== Agent Decision Logic ===")
    for output in agent_outputs:
        r = guard.verify(output)
        if r.ecs >= 70:
            action = "ACCEPT"
        elif r.ecs >= 40:
            action = "REVIEW"
        else:
            action = "REJECT"

        print(f"  {action:>6s} (ECS={r.ecs:5.1f}, prov={r.provenance}): {output[:55]}")
    print()


if __name__ == "__main__":
    example_basic_verification()
    example_threshold_levels()
    example_arithmetic_repair()
    example_degradation_tracking()
    example_decision_logic()
