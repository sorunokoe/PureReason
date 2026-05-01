"""Verification helpers and verify_chain for vCoT reasoning."""

from __future__ import annotations

import re
import sys

from .._core import _run
from .models import EpistemicChainReport, StepVerification

# ---------------------------------------------------------------------------
# Arithmetic error detection (S47)
# ---------------------------------------------------------------------------

_ARITH_RE = re.compile(
    r"(-?\d+(?:\.\d+)?)\s*"
    r"([+\-×x\*÷/])\s*"
    r"(-?\d+(?:\.\d+)?)\s*=\s*"
    r"(-?\d+(?:\.\d+)?)"
)
_PCT_RE = re.compile(
    r"(-?\d+(?:\.\d+)?)\s*%\s+of\s+(-?\d+(?:\.\d+)?)\s+(?:is|=)\s+(-?\d+(?:\.\d+)?)",
    re.IGNORECASE,
)


def _arithmetic_error_in_text(text: str) -> bool:
    """Return True if text contains a verifiable arithmetic error (S47)."""
    for m in _ARITH_RE.finditer(text):
        a, op, b, claimed = (
            float(m.group(1)),
            m.group(2).lower(),
            float(m.group(3)),
            float(m.group(4)),
        )
        if op == "+":
            expected = a + b
        elif op == "-":
            expected = a - b
        elif op in ("*", "x", "×"):
            expected = a * b
        elif op in ("/", "÷"):
            if b == 0:
                continue
            expected = a / b
        else:
            continue
        # Tolerance: 0.5% relative with a 1e-6 absolute floor. The previous
        # 0.01 floor silently accepted small-magnitude errors — e.g. it would
        # not flag "0.001 + 0.001 = 0.002" even though the error is 50%
        # (TRIZ-42 NE-11).
        tol = max(1e-6, abs(expected) * 0.005)
        if abs(expected - claimed) > tol:
            return True
    for m in _PCT_RE.finditer(text):
        pct, base, claimed = float(m.group(1)), float(m.group(2)), float(m.group(3))
        expected = pct * base / 100
        # Tolerance: 0.5% relative with a 1e-6 absolute floor. The previous
        # 0.01 floor silently accepted small-magnitude errors — e.g. it would
        # not flag "0.001 + 0.001 = 0.002" even though the error is 50%
        # (TRIZ-42 NE-11).
        tol = max(1e-6, abs(expected) * 0.005)
        if abs(expected - claimed) > tol:
            return True
    return False


# ---------------------------------------------------------------------------
# Core verification helpers
# ---------------------------------------------------------------------------


def _ecs_for_text(text: str) -> tuple[int, list[str]]:
    """Run pure-reason calibrate on a single text, return (ecs, flags)."""
    try:
        result = _run(["calibrate", text])
        ecs = int(result.get("ecs", 50))
        flags = result.get("flags", [])
        return ecs, flags
    except Exception:
        return 50, []


def _kac_step_vs_context(context: str, step: str) -> tuple[bool, list[str]]:
    """Check if *step* contradicts *context* using KAC (analyze command).

    Returns (is_consistent, flags).  is_consistent=True means no contradiction.
    """
    text = f"Knowledge: {context}\nQuestion: Does the following follow?\nAnswer: {step}"
    try:
        result = _run(["analyze", text])
        has_kac = result.get("verdict", {}).get("has_contradictions", False)
        flags = result.get("flags", [])
        return not has_kac, flags
    except Exception:
        return True, []


# ---------------------------------------------------------------------------
# Public API: verify_chain
# ---------------------------------------------------------------------------


def verify_chain(
    problem: str,
    steps: list[str],
    *,
    verbose: bool = False,
) -> EpistemicChainReport:
    """Verify an existing reasoning chain step-by-step.

    Parameters
    ----------
    problem:
        The original question / problem statement.
    steps:
        Ordered list of reasoning steps (sentences or paragraphs).
        The last step is treated as the answer/conclusion.
    verbose:
        If True, print progress to stderr.

    Returns
    -------
    EpistemicChainReport with per-step verdicts and overall confidence.
    """
    if not steps:
        return EpistemicChainReport(
            problem=problem,
            steps=[],
            answer=None,
            is_valid=False,
            chain_confidence=0.0,
            invalid_steps=[],
            summary="No steps provided.",
        )

    verified_steps: list[StepVerification] = []
    cumulative_context = problem

    for i, step in enumerate(steps):
        if verbose:
            print(f"  Verifying step {i + 1}/{len(steps)}: {step[:60]}...", file=sys.stderr)

        # 1. Internal check: calibrate the step text alone
        ecs, flags = _ecs_for_text(step)
        is_internal = ecs >= 30 and "CERTAINTY_OVERREACH" not in str(flags).upper()

        # 1b. Arithmetic oracle (S47): detect explicit computation errors
        if _arithmetic_error_in_text(step):
            is_internal = False
            flags = [*list(flags), "ARITHMETIC_ERROR"]

        # 2. Contextual check: does this step contradict accumulated context?
        is_contextual = True
        contradiction_with = None
        ctx_flags: list[str] = []

        if cumulative_context:
            is_contextual, ctx_flags = _kac_step_vs_context(cumulative_context, step)
            if not is_contextual:
                for j, prior_sv in enumerate(verified_steps):
                    ok, _ = _kac_step_vs_context(prior_sv.step_text, step)
                    if not ok:
                        contradiction_with = j
                        break

        all_flags = list(set(flags + ctx_flags))
        sv = StepVerification(
            step_index=i,
            step_text=step,
            ecs=ecs,
            is_internally_valid=is_internal,
            is_contextually_valid=is_contextual,
            flags=all_flags,
            contradiction_with_step=contradiction_with,
        )
        verified_steps.append(sv)

        # Accumulate context only from valid steps (don't let errors propagate)
        if is_internal and is_contextual:
            cumulative_context = cumulative_context + " " + step

    # Aggregate
    invalid = [
        sv.step_index
        for sv in verified_steps
        if not sv.is_internally_valid or not sv.is_contextually_valid
    ]
    is_valid = len(invalid) == 0

    ecs_values = [sv.ecs for sv in verified_steps]
    if ecs_values:
        chain_confidence = len(ecs_values) / sum(1 / max(v, 1) for v in ecs_values) / 100
    else:
        chain_confidence = 0.0

    answer = steps[-1] if steps else None

    if is_valid:
        summary = (
            f"All {len(steps)} reasoning steps passed verification. "
            f"Chain confidence: {chain_confidence:.2f}."
        )
    else:
        bad = invalid[0]
        sv0 = verified_steps[bad]
        summary = (
            f"Step {bad + 1} failed: "
            + ("internal consistency violation. " if not sv0.is_internally_valid else "")
            + (
                f"contradicts step {sv0.contradiction_with_step + 1}. "
                if sv0.contradiction_with_step is not None
                else "contextual contradiction detected. "
                if not sv0.is_contextually_valid
                else ""
            )
            + f"Chain confidence: {chain_confidence:.2f}."
        )

    return EpistemicChainReport(
        problem=problem,
        steps=verified_steps,
        answer=answer,
        is_valid=is_valid,
        chain_confidence=chain_confidence,
        invalid_steps=invalid,
        summary=summary,
    )
