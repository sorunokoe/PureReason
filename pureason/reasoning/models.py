"""Data models for the vCoT reasoning engine."""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass
class StepVerification:
    """Verification result for a single reasoning step."""

    step_index: int
    step_text: str
    ecs: int  # 0-100 epistemic confidence score
    is_internally_valid: bool  # no antinomies / paralogisms in isolation
    is_contextually_valid: bool  # consistent with all prior steps
    flags: list[str] = field(default_factory=list)
    contradiction_with_step: int | None = None  # index of conflicting prior step


@dataclass
class EpistemicChainReport:
    """Full verification report for a reasoning chain."""

    problem: str
    steps: list[StepVerification]
    answer: str | None
    is_valid: bool  # all steps pass both checks
    chain_confidence: float  # harmonic mean of step ECS values / 100
    invalid_steps: list[int]
    summary: str

    @property
    def first_failure(self) -> StepVerification | None:
        for sv in self.steps:
            if not sv.is_internally_valid or not sv.is_contextually_valid:
                return sv
        return None
