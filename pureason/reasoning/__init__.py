"""Verified Chain-of-Thought (vCoT) reasoning engine.

PureReason as a reasoning verifier, not just a hallucination detector.
"""

from .arithmetic import solve_arithmetic
from .chain import _ecs_for_text, _kac_step_vs_context, verify_chain
from .mcq import pick_best_answer
from .models import EpistemicChainReport, StepVerification
from .repair import (
    _extract_letter_answer,
    _extract_numeric_answer,
    _majority_vote,
    _majority_vote_letters,
    _repair_arithmetic_in_step,
)
from .syllogism import verify_syllogism

__all__ = [
    "EpistemicChainReport",
    "StepVerification",
    "_ecs_for_text",
    "_extract_letter_answer",
    "_extract_numeric_answer",
    "_kac_step_vs_context",
    "_majority_vote",
    "_majority_vote_letters",
    "_repair_arithmetic_in_step",
    "pick_best_answer",
    "solve_arithmetic",
    "verify_chain",
    "verify_syllogism",
]
