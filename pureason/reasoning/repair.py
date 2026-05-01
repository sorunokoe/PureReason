"""Repair and extraction utilities for vCoT reasoning."""

from __future__ import annotations

import contextlib
import re
from collections import Counter


def _repair_arithmetic_in_step(step_text: str) -> str:
    """Find 'A op B = C' patterns in a step and fix any arithmetic errors.

    This is PureReason's core genuine advantage over raw LLMs: formal
    arithmetic verification + repair (TRIZ P22 Turn Harm into Benefit —
    LLM arithmetic mistakes become opportunities for formal correction).
    """
    _ARITH_PAT = re.compile(
        r"(-?[\d,]+(?:\.\d+)?)\s*([+\-×x÷*/])\s*(-?[\d,]+(?:\.\d+)?)"
        r"\s*=\s*(-?[\d,]+(?:\.\d+)?)",
    )

    def _fix(m: re.Match[str]) -> str:
        a_s, op, b_s, c_s = m.group(1), m.group(2), m.group(3), m.group(4)
        try:
            a = float(a_s.replace(",", ""))
            b = float(b_s.replace(",", ""))
            c_claimed = float(c_s.replace(",", ""))
        except ValueError:
            return m.group(0)
        op_results = {
            "+": a + b,
            "-": a - b,
            "×": a * b,
            "x": a * b,
            "*": a * b,
            "÷": a / b if b != 0 else None,
            "/": a / b if b != 0 else None,
        }
        correct = op_results.get(op)
        if correct is None:
            return m.group(0)
        if abs(correct - c_claimed) > max(1e-6, 0.001 * abs(correct)):
            if correct == int(correct):
                correct_s = str(int(correct))
            else:
                correct_s = f"{correct:.4g}"
            return m.group(0).replace(f"= {c_s}", f"= {correct_s} [repaired]", 1)
        return m.group(0)

    return _ARITH_PAT.sub(_fix, step_text)


def _extract_numeric_answer(text: str) -> float | None:
    """Extract the final numeric answer from a chain / reply text."""
    m = re.search(
        r"(?:the answer is|answer:|therefore,?)\s*\$?(-?[\d,]+(?:\.\d+)?)",
        text,
        re.IGNORECASE,
    )
    if m:
        try:
            return float(m.group(1).replace(",", ""))
        except ValueError:
            pass
    tail = text[-200:]
    nums = re.findall(r"\b(-?[\d,]+(?:\.\d+)?)\b", tail)
    for n in reversed(nums):
        try:
            return float(n.replace(",", ""))
        except ValueError:
            continue
    return None


def _extract_letter_answer(text: str) -> str | None:
    """Extract the final letter answer (A-D) from an MCQ chain reply text.

    Returns the letter as a single uppercase character, or None.
    """
    text = text.strip()
    tail = text[-300:] if len(text) > 300 else text
    for pattern in [
        r"[Tt]herefore[,\s]+the answer is\s+([A-D])\b",
        r"[Tt]he (?:correct |best )?answer is\s+([A-D])\b",
        r"ANSWER:\s*([A-D])\b",
        r"\*\*([A-D])\*\*",
        r"answer[:\s]+\*\*([A-D])\*\*",
        r"\banswer\b.*?\b([A-D])\b[^A-Za-z]",
        r"\b([A-D])\s+is\s+(?:correct|the answer)",
    ]:
        m = re.search(pattern, tail, re.IGNORECASE)
        if m:
            return m.group(1).upper()
    letters = list(re.finditer(r"\b([A-D])\b", text))
    if letters:
        return letters[-1].group(1).upper()
    return None


def _majority_vote_letters(answers: list) -> str | None:
    """Return most common non-None letter answer."""
    valid = [a for a in answers if a is not None]
    if not valid:
        return None
    return Counter(valid).most_common(1)[0][0]


def _majority_vote(answers: list) -> float | None:
    """Return the most common non-None float answer (rounded to 2dp)."""
    valid = []
    for a in answers:
        if a is not None:
            with contextlib.suppress(TypeError, ValueError):
                valid.append(round(float(a), 2))
    if not valid:
        return None
    return Counter(valid).most_common(1)[0][0]
