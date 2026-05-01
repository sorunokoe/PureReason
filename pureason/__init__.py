"""PureReason — zero-parameter epistemic calibration for AI outputs.

Quick start::

    from pureason import calibrate, analyze

    # ECS score for any text
    r = calibrate("The patient must have cancer.")
    print(r.ecs, r.band, r.flags)

    # Hallucination check with a reference document
    r = analyze("The Earth orbits the Moon.", reference="The Earth orbits the Sun.")
    print(r.hallucination_detected, r.kac_score)

The binary (``pure-reason``) must be on PATH or set via the ``PUREASON_BINARY``
environment variable.  Build it with ``cargo build --release`` inside the repo.
"""

from __future__ import annotations

from ._core import _run
from ._models import AnalysisResult, CalibrationResult, HallucinationFlag, ScoreBreakdown
from .reasoning import (
    EpistemicChainReport,
    StepVerification,
    pick_best_answer,
    solve_arithmetic,
    verify_chain,
    verify_syllogism,
)


def calibrate(text: str) -> CalibrationResult:
    """Compute the Epistemic Confidence Score (ECS) for *text*.

    Parameters
    ----------
    text:
        The text to evaluate.  Can be any natural-language string.

    Returns
    -------
    CalibrationResult
        Contains ``.ecs`` (0–100), ``.band`` (Low/Moderate/High),
        ``.calibrated``, ``.epistemic_mode``, ``.flags``, and
        ``.safe_version``.
    """
    data = _run(["calibrate", text])
    return _parse_calibration(data)


def analyze(
    text: str,
    reference: str | None = None,
    question: str | None = None,
) -> AnalysisResult:
    """Full Kantian pipeline analysis with optional hallucination detection.

    When *reference* is provided the engine checks whether *text* contradicts
    the reference document using the KAC (Knowledge-Answer Contradiction)
    detector.

    Parameters
    ----------
    text:
        The AI-generated answer or statement to evaluate.
    reference:
        Optional grounding document / knowledge source.  When supplied, the
        input is formatted as ``Knowledge: {reference}\\nAnswer: {text}`` so
        that entity-novelty and KAC detection engage.
    question:
        Optional question that was posed (included in the formatted input when
        *reference* is also provided).

    Returns
    -------
    AnalysisResult
        Superset of CalibrationResult — additionally exposes
        ``.hallucination_detected``, ``.kac_score``, ``.entity_novelty``,
        and ``.hallucination_flags``.
    """
    if reference is not None:
        if question is not None:
            stdin_text = f"Knowledge: {reference}\nQuestion: {question}\nAnswer: {text}"
        else:
            stdin_text = f"Knowledge: {reference}\nAnswer: {text}"
        data = _run(["analyze"], stdin_text=stdin_text)
    else:
        data = _run(["analyze", text])

    return _parse_analysis(data)


def flags(text: str, reference: str | None = None) -> list[str]:
    """Return only the list of epistemic flag strings for *text*.

    Convenience wrapper around :func:`analyze` for callers that only need to
    know whether flags were raised without the full result object.
    """
    result = analyze(text, reference=reference)
    flag_list = list(result.flags)
    if result.hallucination_detected:
        flag_list.append("hallucination")
    return flag_list


# ─── Internal parsers ─────────────────────────────────────────────────────────


def _parse_calibration(data: dict) -> CalibrationResult:
    breakdown = ScoreBreakdown.from_dict(data.get("score_breakdown", {}))
    return CalibrationResult(
        ecs=int(data.get("ecs", 50)),
        band=str(data.get("band", "Moderate")),
        calibrated=bool(data.get("calibrated", False)),
        epistemic_mode=str(data.get("epistemic_mode", "")),
        flags=list(data.get("flags", [])),
        safe_version=str(data.get("safe_version", "")),
        score_breakdown=breakdown,
    )


def _parse_analysis(data: dict) -> AnalysisResult:
    # The `analyze` command returns a richer structure than `calibrate`.
    # ECS/band/flags may be nested under a "calibration" key or at the top
    # level depending on the CLI version.
    cal_data = data.get("calibration", data)
    breakdown = ScoreBreakdown.from_dict(cal_data.get("score_breakdown", {}))

    dialectic = data.get("dialectic", {})
    kac_score: float | None = dialectic.get("kac_score")
    entity_novelty: float | None = dialectic.get("entity_novelty")

    hallucination_flags: list[HallucinationFlag] = []
    hallucination_detected = False

    for illusion in dialectic.get("illusions", []):
        description = illusion.get("description", "")
        severity = illusion.get("severity", "Low")
        kind = illusion.get("source", illusion.get("kind", "Unknown"))
        hallucination_flags.append(
            HallucinationFlag(kind=str(kind), description=description, severity=severity)
        )

    for antinomy in dialectic.get("antinomies", []):
        if antinomy.get("has_conflict"):
            hallucination_detected = True
            desc = antinomy.get("description", "Antinomy detected.")
            hallucination_flags.append(
                HallucinationFlag(kind="KAC", description=desc, severity="High")
            )

    if kac_score is not None and kac_score > 0.3:
        hallucination_detected = True

    if entity_novelty is not None and entity_novelty > 0.4:
        hallucination_detected = True

    return AnalysisResult(
        ecs=int(cal_data.get("ecs", 50)),
        band=str(cal_data.get("band", "Moderate")),
        calibrated=bool(cal_data.get("calibrated", False)),
        epistemic_mode=str(cal_data.get("epistemic_mode", "")),
        flags=list(cal_data.get("flags", [])),
        safe_version=str(cal_data.get("safe_version", "")),
        score_breakdown=breakdown,
        hallucination_detected=hallucination_detected,
        kac_score=kac_score,
        entity_novelty=entity_novelty,
        hallucination_flags=hallucination_flags,
    )


__all__ = [
    "AnalysisResult",
    "CalibrationResult",
    "EpistemicChainReport",
    "HallucinationFlag",
    "ScoreBreakdown",
    "StepVerification",
    "analyze",
    "calibrate",
    "flags",
    "pick_best_answer",
    "solve_arithmetic",
    "verify_chain",
    "verify_syllogism",
]
