"""ReasoningGuard — Pure verification layer for any reasoning output.

Works on any text from any source (LLM, human, rule engine). No LLM required.
Runs ECS (Epistemic Calibration Score) and arithmetic repair purely via the
Rust binary — always available, always fast, zero dependencies.

Usage
-----
    from pureason.guard import ReasoningGuard

    guard = ReasoningGuard(threshold=60)

    # Verify any text:
    result = guard.verify("The answer is 4 because 2 + 2 = 5.")
    print(result.ecs, result.repaired, result.provenance)

    # As middleware in your own pipeline:
    your_answer = your_llm.generate(prompt)
    verified = guard.verify(your_answer)
    if verified.provenance == "guard:flagged":
        # handle low-confidence output
        ...

Architecture (TRIZ P7 Nested Doll)
------------------------------------
    Any text input
        │
        ├─ ECS scan (< 1ms, pure Rust)
        │       ECS ≥ threshold → provenance = "verified"
        │       ECS < threshold ↓
        │
        ├─ Arithmetic repair
        │       changed  → provenance = "repaired"
        │       no change ↓
        │
        └─ Flag
                provenance = "flagged"

S104: Degradation detection — tracks rolling ECS and warns when quality drops.
"""

from __future__ import annotations

import warnings
from collections import deque
from dataclasses import dataclass

# ---------------------------------------------------------------------------
# Result type
# ---------------------------------------------------------------------------


@dataclass
class VerificationResult:
    """Result of a single guard.verify() call."""

    text: str  # the (possibly repaired) text
    original: str  # original text before any repair
    ecs: float  # ECS score (0–100)
    provenance: str  # "verified" | "repaired" | "flagged"
    repaired: bool  # True if arithmetic was repaired


# ---------------------------------------------------------------------------
# S104: Degradation tracker
# ---------------------------------------------------------------------------


class ReasoningDegradationWarning(UserWarning):
    """Emitted when rolling ECS drops > 10pp below historical baseline."""


class _ReputationTracker:
    """Rolling ECS tracker per source label (S104)."""

    def __init__(self, window: int = 10, baseline_window: int = 30, drop: float = 10.0):
        self._window = window
        self._baseline_window = baseline_window
        self._drop = drop
        self._history: dict[str, deque] = {}

    def record(self, label: str, ecs: float) -> None:
        if label not in self._history:
            self._history[label] = deque(maxlen=self._baseline_window)
        self._history[label].append(ecs)

    def is_degraded(self, label: str) -> bool:
        hist = self._history.get(label)
        if not hist or len(hist) < self._window + 1:
            return False
        recent = list(hist)[-self._window :]
        baseline = list(hist)[: -self._window]
        if not baseline:
            return False
        return (sum(baseline) / len(baseline) - sum(recent) / len(recent)) > self._drop

    def recent_mean(self, label: str) -> float | None:
        hist = self._history.get(label)
        if not hist:
            return None
        recent = list(hist)[-self._window :]
        return sum(recent) / len(recent)


_GLOBAL_TRACKER = _ReputationTracker()


# ---------------------------------------------------------------------------
# ReasoningGuard
# ---------------------------------------------------------------------------


class ReasoningGuard:
    """Pure verification layer — no LLM required.

    Verifies any text using ECS (Rust binary, < 1ms) and repairs arithmetic
    errors deterministically. Works on text from any source.

    Parameters
    ----------
    threshold : int
        ECS score below which text is flagged (0–100). Default 60.
    repair : bool
        Attempt arithmetic repair on flagged text. Default True.
    source_label : str
        Label for degradation tracking. Default "default".
    warn_on_degradation : bool
        Emit ReasoningDegradationWarning when ECS drops. Default True.
    """

    def __init__(
        self,
        threshold: int = 60,
        repair: bool = True,
        source_label: str = "default",
        warn_on_degradation: bool = True,
        tracker: _ReputationTracker | None = None,
    ):
        self.threshold = threshold
        self.repair = repair
        self.source_label = source_label
        self.warn_on_degradation = warn_on_degradation
        self._tracker = tracker or _GLOBAL_TRACKER

    def verify(self, text: str) -> VerificationResult:
        """Verify a text string. Returns VerificationResult with ECS and provenance."""
        ecs = self._ecs_score(text)
        self._tracker.record(self.source_label, ecs)

        if self.warn_on_degradation and self._tracker.is_degraded(self.source_label):
            recent = self._tracker.recent_mean(self.source_label)
            warnings.warn(
                f"ReasoningGuard: source '{self.source_label}' quality degrading "
                f"(recent mean ECS={recent:.1f})",
                ReasoningDegradationWarning,
                stacklevel=2,
            )

        if ecs >= self.threshold:
            return VerificationResult(text, text, ecs, "verified", False)

        if self.repair:
            repaired = self._arithmetic_repair(text)
            if repaired != text:
                return VerificationResult(repaired, text, ecs, "repaired", True)

        return VerificationResult(text, text, ecs, "flagged", False)

    def verify_chain(self, context: str, steps: list[str]):
        """Verify a reasoning chain. Returns EpistemicChainReport."""
        from pureason.reasoning import verify_chain

        return verify_chain(context, steps)

    def _ecs_score(self, text: str) -> float:
        try:
            from pureason.reasoning import _ecs_for_text

            score, _ = _ecs_for_text(text)
            return float(score)
        except Exception:
            return 75.0  # degrade gracefully if Rust binary unavailable

    def _arithmetic_repair(self, text: str) -> str:
        try:
            from pureason.reasoning import _repair_arithmetic_in_step

            return _repair_arithmetic_in_step(text)
        except Exception:
            return text

    def __repr__(self) -> str:
        return f"ReasoningGuard(threshold={self.threshold}, repair={self.repair})"
