"""Multiple-choice question picker for vCoT reasoning."""

from __future__ import annotations

from .chain import verify_chain
from .models import EpistemicChainReport


class AmbiguousAnswerError(RuntimeError):
    """Raised by ``pick_best_answer(..., strict=True)`` when ECS is tied.

    The MCQ picker used to silently return index 0 on ties (TRIZ-42 NE-10),
    which masked the fact that the ECS signal cannot distinguish the choices.
    In ``strict`` mode we surface the ambiguity instead of guessing.
    """

    def __init__(self, tied_indices: list[int], ecs: int):
        super().__init__(f"ECS cannot distinguish choices {tied_indices} (all scored {ecs})")
        self.tied_indices = tied_indices
        self.ecs = ecs


def pick_best_answer(
    question: str,
    choices: list[str],
    *,
    context: str = "",
    strict: bool = False,
) -> tuple[int, EpistemicChainReport]:
    """Return the index of the best answer by ECS-based verification.

    Parameters
    ----------
    question, choices, context:
        See prior version.
    strict:
        If ``True`` and two or more choices tie for the top ECS, raise
        :class:`AmbiguousAnswerError` instead of silently picking the first.
        If ``False`` (default, for backward compatibility), the chosen
        report's flags include ``MCQ_AMBIGUOUS_ECS_TIE`` so callers can detect
        the situation and degrade gracefully.

    Returns
    -------
    (best_index, report_for_best_choice)
    """
    if not choices:
        raise ValueError("pick_best_answer requires at least one choice")

    prefix = f"{context}\n{question}" if context else question

    scored: list[tuple[int, int, EpistemicChainReport]] = []
    for i, choice in enumerate(choices):
        report = verify_chain(prefix, [choice])
        ecs = report.steps[0].ecs if report.steps else 0
        scored.append((i, ecs, report))

    best_ecs = max(ecs for _, ecs, _ in scored)
    tied = [i for i, ecs, _ in scored if ecs == best_ecs]

    if len(tied) == 1:
        best_idx = tied[0]
        _, _, best_report = scored[best_idx]
        return best_idx, best_report

    if strict:
        raise AmbiguousAnswerError(tied, best_ecs)

    # Non-strict: return the lowest tied index but annotate the report so
    # upstream code can tell the ECS did not actually discriminate.
    best_idx = tied[0]
    _, _, best_report = scored[best_idx]
    if best_report.steps:
        step = best_report.steps[0]
        if "MCQ_AMBIGUOUS_ECS_TIE" not in step.flags:
            step.flags.append("MCQ_AMBIGUOUS_ECS_TIE")
    return best_idx, best_report
