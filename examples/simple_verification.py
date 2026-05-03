#!/usr/bin/env python3
"""Simple example: Verify a single claim with PureReason.

This example shows the most basic usage - verify a claim and check its score.
"""

import sys

sys.path.insert(0, ".")

from pureason.guard import ReasoningGuard


def main():
    """Verify simple claims."""

    # Create a guard with threshold=60 (moderate strictness)
    guard = ReasoningGuard(threshold=60)

    # Example 1: Factual claim (should pass)
    claim1 = "Water boils at 100°C at sea level."
    print("=" * 60)
    print(f"Verifying: {claim1}")
    print("=" * 60)

    result1 = guard.verify(claim1)
    print(f"ECS Score: {result1.ecs:.1f}/100")
    print(f"Provenance: {result1.provenance}")
    print(f"Status: {'VERIFIED' if result1.ecs >= 60 else 'FLAGGED'}")
    print()

    # Example 2: Arithmetic error (should auto-repair)
    claim2 = "The answer is 4 because 2 + 2 = 5."
    print("=" * 60)
    print(f"Verifying: {claim2}")
    print("=" * 60)

    result2 = guard.verify(claim2)
    print(f"ECS Score: {result2.ecs:.1f}/100")
    print(f"Provenance: {result2.provenance}")
    if result2.repaired:
        print(f"Original: {result2.original}")
        print(f"Repaired: {result2.text}")
    print()

    # Example 3: Low-confidence claim (should flag)
    claim3 = "The temperature is both hot and cold simultaneously."
    print("=" * 60)
    print(f"Verifying: {claim3}")
    print("=" * 60)

    result3 = guard.verify(claim3)
    print(f"ECS Score: {result3.ecs:.1f}/100")
    print(f"Provenance: {result3.provenance}")
    print(f"Status: {'VERIFIED' if result3.ecs >= 60 else 'FLAGGED'}")
    print()

    # Decision logic for AI agents
    print("=" * 60)
    print("AGENT DECISION LOGIC")
    print("=" * 60)
    for i, (claim, result) in enumerate([
        (claim1, result1), (claim2, result2), (claim3, result3)
    ], 1):
        if result.ecs >= 70:
            decision = "✅ ACCEPT"
        elif result.ecs >= 40:
            decision = "⚠️  REVIEW"
        else:
            decision = "❌ REJECT"
        print(f"{i}. {decision} (ECS: {result.ecs:.1f}) - {claim[:50]}...")


if __name__ == "__main__":
    main()
