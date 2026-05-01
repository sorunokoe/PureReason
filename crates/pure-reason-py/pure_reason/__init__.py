"""
PureReason: Kant's Critique of Pure Reason as a Python reasoning library.

This package provides epistemic validation for LLM outputs using the Kantian
cognitive framework. Import the PureReason class or use the convenience functions.

Installation:
    pip install pure-reason

Quick start:
    from pure_reason import validate, Pipeline
    result = validate("Every event has a cause.")
    print(result["risk_level"])  # "SAFE"
"""

from .exceptions import EpistemicViolationError, PureReasonError
from .pipeline import Pipeline, certify, regulate, validate

__version__ = "0.1.0"
__all__ = [
    "EpistemicViolationError",
    "Pipeline",
    "PureReasonError",
    "certify",
    "regulate",
    "validate",
]
