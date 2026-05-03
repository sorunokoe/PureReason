#!/usr/bin/env python3
"""LangChain integration example: Verify LLM outputs with PureReason.

This shows how to use PureReason as a verification layer in a LangChain pipeline.
Install: pip install langchain langchain-openai
"""

import sys

sys.path.insert(0, ".")

from pureason.guard import ReasoningGuard


class PureReasonVerifier:
    """LangChain-compatible verifier using PureReason.

    Usage:
        verifier = PureReasonVerifier(min_ecs=70)
        result = verifier.verify(llm_output)
        if result['passed']:
            # Use the output
        else:
            # Handle verification failure
    """

    def __init__(self, min_ecs: int = 70, auto_repair: bool = True):
        """Initialize verifier.

        Args:
            min_ecs: Minimum epistemic confidence score (0-100)
            auto_repair: If True, automatically repair arithmetic errors
        """
        self.min_ecs = min_ecs
        self.guard = ReasoningGuard(threshold=min_ecs, repair=auto_repair)

    def verify(self, text: str) -> dict:
        """Verify a text string.

        Returns:
            dict with keys:
                - passed (bool): True if ECS >= min_ecs
                - ecs (float): Epistemic confidence score
                - provenance (str): "verified", "repaired", or "flagged"
                - repaired (bool): True if arithmetic was fixed
                - text (str): Final text (possibly repaired)
                - original (str): Original input text
        """
        result = self.guard.verify(text)

        return {
            "passed": result.ecs >= self.min_ecs,
            "ecs": result.ecs,
            "provenance": result.provenance,
            "repaired": result.repaired,
            "text": result.text,
            "original": result.original,
        }


def example_without_langchain():
    """Example: Verify LLM-style outputs directly."""
    print("=" * 70)
    print("EXAMPLE 1: Direct Verification")
    print("=" * 70)

    verifier = PureReasonVerifier(min_ecs=70)

    # Simulate LLM outputs
    llm_outputs = [
        "Based on the symptoms, the patient likely has the flu.",
        "The answer is 25 because 10 + 15 = 26.",
        "The solution is X, but also not X, depending on the context.",
    ]

    for output in llm_outputs:
        result = verifier.verify(output)
        status = "✅ PASS" if result["passed"] else "❌ FAIL"
        print(f"\n{status} (ECS: {result['ecs']:.1f}/100)")
        print(f"Text: {output}")
        print(f"Provenance: {result['provenance']}")
        if result["repaired"]:
            print(f"Original: {result['original']}")
            print(f"Repaired: {result['text']}")


def example_with_langchain():
    """Example: Use PureReason in a LangChain chain."""
    try:
        from langchain.prompts import PromptTemplate
    except ImportError:
        print("\n❌ LangChain not installed. Install with:")
        print("   pip install langchain langchain-openai")
        return

    print("\n" + "=" * 70)
    print("EXAMPLE 2: LangChain Integration")
    print("=" * 70)

    # Create a simple chain
    template = "Diagnose the patient with these symptoms: {symptoms}"
    PromptTemplate(template=template, input_variables=["symptoms"])

    # Note: Requires OPENAI_API_KEY environment variable
    # For demo purposes, we'll simulate this
    print("\nSimulated LangChain workflow:")
    print("1. LLM generates diagnosis")
    print("2. PureReason verifies output")
    print("3. If verification fails, use repaired output or retry")

    verifier = PureReasonVerifier(min_ecs=70)

    # Simulate LLM response
    symptoms = "fever, cough, fatigue"
    llm_response = "The patient has symptoms consistent with influenza."

    print(f"\nInput: {symptoms}")
    print(f"LLM Output: {llm_response}")

    verification = verifier.verify(llm_response)
    print(f"\nVerification: {'PASS' if verification['passed'] else 'FAIL'}")
    print(f"ECS: {verification['ecs']:.1f}/100")
    print(f"Provenance: {verification['provenance']}")

    if verification["repaired"]:
        print("\n✓ Output was automatically repaired:")
        print(f"   Original: {verification['original']}")
        print(f"   Repaired: {verification['text']}")


def main():
    """Run all examples."""
    example_without_langchain()
    example_with_langchain()

    print("\n" + "=" * 70)
    print("INTEGRATION TIPS")
    print("=" * 70)
    print("""
1. Set min_ecs based on your risk tolerance:
   - High risk (medical, legal): 80+
   - Medium risk (business): 70+
   - Low risk (creative): 60+

2. Use rewrites to improve LLM outputs:
   - If verification fails, use the rewrite
   - Or re-prompt the LLM with feedback

3. Monitor ECS scores over time:
   - Track average ECS per model
   - A/B test different prompts
   - Identify problematic domains

4. Batch verification for efficiency:
   - Verify multiple outputs together
   - Use async/parallel processing
    """)


if __name__ == "__main__":
    main()
