"""Verdict functions.

TRIZ-42 NE-4 remediation
------------------------
Previously this module exposed one bespoke verdict function per benchmark
(``halueval_qa_combined_verdict``, ``verdict_is_issue_felm``,
``truthfulqa_combined_verdict``, ``grounded_combined_verdict``, ...) — each
with its own thresholds and rule combinations.  Those were calibrated on the
same ``seed=42`` draw they were evaluated on, which meant the reported F1 was
dataset-specific tuning dressed up as a general detector.

This revision collapses them to a single :func:`universal_verdict` applied
uniformly across every benchmark.  The per-benchmark names are kept as thin
aliases so callers do not break, but they all route through the same rule.
New benchmarks MUST use :func:`universal_verdict` directly.
"""

from .felm_oracles import arithmetic_error_oracle, reasoning_chain_consistency_oracle
from .semantic import _entity_novelty_grounded, _unigram_faithfulness


def claims_verdict_is_issue(verdict: dict) -> bool:
    """Claims-level issue detection: flag if any claim is risky (TRIZ P1 OR rule)."""
    risk = str(verdict.get("risk", "Safe")).lower()
    return (
        verdict.get("risky_claims", 0) > 0
        or verdict.get("has_illusions", False)
        or verdict.get("has_contradictions", False)
        or verdict.get("has_paralogisms", False)
        or risk in ("medium", "high")
    )


# ─── Universal verdict ───────────────────────────────────────────────────────


def universal_verdict(text: str, verdict: dict) -> bool:
    """Benchmark-agnostic issue classifier.

    Combines the benchmark-general signals produced by ``pure-reason analyze``:

    * ``has_contradictions`` (KAC): structurally reliable — always flags.
    * ``has_illusions`` (world-prior / entity-novelty overreach): flags unless
      the answer has high unigram faithfulness to its context, which suggests
      the prior match was accidental.
    * ``has_paralogisms``: flags.
    * Arithmetic / reasoning-chain self-consistency oracles fire on
      intrinsically checkable text (math, quantity drift).
    * Entity novelty against a grounding block fires when the text introduces
      entities absent from its own context.

    Every signal is a property of the text and the engine's output — none of
    them read any benchmark ground-truth file. The same rule is applied to
    every benchmark; any observed F1 differences reflect actual detector
    generality, not tuning.
    """
    if verdict.get("has_contradictions", False):
        return True
    if verdict.get("has_illusions", False):
        # Faithful answers that happen to share phrasing with a world prior
        # (e.g. a correct debunking paragraph) would otherwise be false
        # positives. Keep the prior match as a flag only when the text adds
        # material that is NOT in its context.
        if _unigram_faithfulness(text) < 0.40:
            return True
    if verdict.get("has_paralogisms", False):
        return True
    if str(verdict.get("risk", "Safe")).lower() == "high":
        return True
    if arithmetic_error_oracle(text):
        return True
    if reasoning_chain_consistency_oracle(text):
        return True
    return bool(_entity_novelty_grounded(text))


# ─── Deprecated per-benchmark aliases ────────────────────────────────────────
# Kept for backward compatibility; all route through ``universal_verdict``.
# New code should not reference these.


def verdict_is_issue_grounded(verdict: dict) -> bool:
    """Deprecated: use :func:`universal_verdict`."""
    return universal_verdict("", verdict)


def grounded_combined_verdict(text: str, verdict: dict) -> bool:
    """Deprecated: use :func:`universal_verdict`."""
    return universal_verdict(text, verdict)


def halueval_qa_combined_verdict(text: str, verdict: dict) -> bool:
    """Deprecated: use :func:`universal_verdict`."""
    return universal_verdict(text, verdict)


def ragtruth_combined_verdict(text: str, verdict: dict) -> bool:
    """Deprecated: use :func:`universal_verdict`."""
    return universal_verdict(text, verdict)


def faithbench_combined_verdict(text: str, verdict: dict) -> bool:
    """Deprecated: use :func:`universal_verdict`."""
    return universal_verdict(text, verdict)


def halueval_dialogue_combined_verdict(text: str, verdict: dict) -> bool:
    """Deprecated: use :func:`universal_verdict`."""
    return universal_verdict(text, verdict)


def verdict_is_issue_ungrounded(verdict: dict) -> bool:
    """Deprecated: use :func:`universal_verdict`."""
    return universal_verdict("", verdict)


def verdict_is_issue_felm(verdict: dict) -> bool:
    """Deprecated: use :func:`universal_verdict`."""
    return universal_verdict("", verdict)


def verdict_is_issue(verdict: dict) -> bool:
    """Deprecated: use :func:`universal_verdict`."""
    return universal_verdict("", verdict)
