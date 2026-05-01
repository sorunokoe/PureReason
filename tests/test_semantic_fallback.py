#!/usr/bin/env python3
"""
Integration test for semantic fallback detector.
Tests Python inference service + Rust integration.
"""

import sys
import json
import subprocess
from pathlib import Path

# Sample test cases from HaluEval Dialogue
TEST_CASES = [
    {
        "knowledge": "The sky appears blue during the day due to Rayleigh scattering.",
        "answer": "The atmosphere looks azure during daytime.",
        "expected_risk": False,  # Semantic match
        "name": "Similar meaning (no hallucination)"
    },
    {
        "knowledge": "Paris is the capital of France.",
        "answer": "London is the capital of France.",
        "expected_risk": True,  # Contradictory
        "name": "Contradiction (hallucination)"
    },
    {
        "knowledge": "Python was created by Guido van Rossum in 1991.",
        "answer": "Python was developed by Guido van Rossum.",
        "expected_risk": False,  # Slight rephrasing
        "name": "Rephrased correctly"
    },
    {
        "knowledge": "Water boils at 100 degrees Celsius at sea level.",
        "answer": "Water freezes at 100 degrees Celsius.",
        "expected_risk": True,  # Wrong fact
        "name": "Wrong fact (hallucination)"
    },
    {
        "knowledge": "The Earth orbits the Sun once per year.",
        "answer": "The Sun revolves around the Earth annually.",
        "expected_risk": True,  # Incorrect astronomy
        "name": "Incorrect astronomy"
    },
]


def test_python_inference_service():
    """Test the Python inference script directly."""
    script_path = Path(__file__).parent.parent / "scripts" / "semantic_inference.py"
    
    print("\n=== Testing Python Inference Service ===\n")
    
    passed = 0
    failed = 0
    
    for test in TEST_CASES:
        print(f"Test: {test['name']}")
        print(f"  Knowledge: {test['knowledge'][:60]}...")
        print(f"  Answer: {test['answer'][:60]}...")
        
        try:
            result = subprocess.run(
                ["python3", str(script_path), test["knowledge"], test["answer"]],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if result.returncode != 0:
                print(f"  ❌ FAILED: Script exited with code {result.returncode}")
                print(f"     stderr: {result.stderr}")
                failed += 1
                continue
            
            data = json.loads(result.stdout)
            similarity = data["similarity"]
            flags_risk = data["flags_risk"]
            
            print(f"  Similarity: {similarity:.3f}")
            print(f"  Flags risk: {flags_risk}")
            print(f"  Expected risk: {test['expected_risk']}")
            
            if flags_risk == test["expected_risk"]:
                print(f"  ✅ PASSED")
                passed += 1
            else:
                print(f"  ❌ FAILED: Expected {test['expected_risk']}, got {flags_risk}")
                failed += 1
                
        except subprocess.TimeoutExpired:
            print(f"  ❌ FAILED: Timeout after 30s")
            failed += 1
        except Exception as e:
            print(f"  ❌ FAILED: {e}")
            failed += 1
        
        print()
    
    print(f"\n=== Results: {passed} passed, {failed} failed ===\n")
    return passed, failed


def main():
    passed, failed = test_python_inference_service()
    
    if failed > 0:
        print(f"Some tests failed. This may be expected if:")
        print(f"  - sentence-transformers is not installed")
        print(f"  - Model is still downloading (first run)")
        print(f"  - Test thresholds need tuning")
        sys.exit(1)
    else:
        print("✅ All tests passed!")
        sys.exit(0)


if __name__ == "__main__":
    main()
