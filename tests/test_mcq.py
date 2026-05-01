"""Tests for pureason.reasoning.mcq — pick_best_answer and AmbiguousAnswerError."""

from __future__ import annotations

import unittest
from unittest.mock import patch


def _make_report(ecs: int, flags: list[str] | None = None):
    """Build an EpistemicChainReport with a single step at the given ECS."""
    from pureason.reasoning import EpistemicChainReport, StepVerification

    step = StepVerification(
        step_index=0,
        step_text="choice text",
        ecs=ecs,
        is_internally_valid=ecs >= 50,
        is_contextually_valid=ecs >= 50,
        flags=list(flags or []),
    )
    return EpistemicChainReport(
        problem="test question",
        steps=[step],
        answer=None,
        is_valid=ecs >= 50,
        chain_confidence=ecs / 100.0,
        invalid_steps=[] if ecs >= 50 else [0],
        summary="test",
    )


class TestPickBestAnswer(unittest.TestCase):
    """Unit tests for pick_best_answer using mocked verify_chain."""

    def test_returns_highest_ecs_index(self) -> None:
        from pureason.reasoning.mcq import pick_best_answer

        reports = [_make_report(60), _make_report(80), _make_report(40)]
        with patch("pureason.reasoning.mcq.verify_chain", side_effect=reports):
            idx, _ = pick_best_answer("Which is best?", ["A", "B", "C"])
        self.assertEqual(idx, 1)

    def test_returns_best_report(self) -> None:
        from pureason.reasoning.mcq import pick_best_answer

        reports = [_make_report(50), _make_report(90)]
        with patch("pureason.reasoning.mcq.verify_chain", side_effect=reports):
            idx, report = pick_best_answer("Q?", ["A", "B"])
        self.assertEqual(idx, 1)
        self.assertEqual(report.steps[0].ecs, 90)

    def test_empty_choices_raises_value_error(self) -> None:
        from pureason.reasoning.mcq import pick_best_answer

        with self.assertRaises(ValueError):
            pick_best_answer("Q?", [])

    def test_single_choice_returns_zero(self) -> None:
        from pureason.reasoning.mcq import pick_best_answer

        reports = [_make_report(75)]
        with patch("pureason.reasoning.mcq.verify_chain", side_effect=reports):
            idx, _ = pick_best_answer("Q?", ["only choice"])
        self.assertEqual(idx, 0)

    def test_tie_non_strict_returns_first_tied_index(self) -> None:
        from pureason.reasoning.mcq import pick_best_answer

        reports = [_make_report(70), _make_report(70), _make_report(50)]
        with patch("pureason.reasoning.mcq.verify_chain", side_effect=reports):
            idx, _ = pick_best_answer("Q?", ["A", "B", "C"])
        self.assertEqual(idx, 0)

    def test_tie_non_strict_adds_ambiguous_flag(self) -> None:
        from pureason.reasoning.mcq import pick_best_answer

        reports = [_make_report(70), _make_report(70)]
        with patch("pureason.reasoning.mcq.verify_chain", side_effect=reports):
            _, report = pick_best_answer("Q?", ["A", "B"])
        self.assertIn("MCQ_AMBIGUOUS_ECS_TIE", report.steps[0].flags)

    def test_no_tie_non_strict_no_ambiguous_flag(self) -> None:
        from pureason.reasoning.mcq import pick_best_answer

        reports = [_make_report(80), _make_report(60)]
        with patch("pureason.reasoning.mcq.verify_chain", side_effect=reports):
            _, report = pick_best_answer("Q?", ["A", "B"])
        self.assertNotIn("MCQ_AMBIGUOUS_ECS_TIE", report.steps[0].flags)

    def test_tie_strict_raises_ambiguous_error(self) -> None:
        from pureason.reasoning.mcq import AmbiguousAnswerError, pick_best_answer

        reports = [_make_report(70), _make_report(70)]
        with (
            patch("pureason.reasoning.mcq.verify_chain", side_effect=reports),
            self.assertRaises(AmbiguousAnswerError) as ctx,
        ):
            pick_best_answer("Q?", ["A", "B"], strict=True)
        self.assertEqual(ctx.exception.ecs, 70)
        self.assertEqual(set(ctx.exception.tied_indices), {0, 1})

    def test_no_tie_strict_mode_works_normally(self) -> None:
        from pureason.reasoning.mcq import pick_best_answer

        reports = [_make_report(60), _make_report(80)]
        with patch("pureason.reasoning.mcq.verify_chain", side_effect=reports):
            idx, _ = pick_best_answer("Q?", ["A", "B"], strict=True)
        self.assertEqual(idx, 1)

    def test_context_prepended_to_question(self) -> None:
        """Context string is prepended when calling verify_chain."""
        from pureason.reasoning.mcq import pick_best_answer

        calls: list[str] = []
        reports = [_make_report(70), _make_report(60)]

        def capture(problem, steps):
            calls.append(problem)
            return reports[len(calls) - 1]

        with patch("pureason.reasoning.mcq.verify_chain", side_effect=capture):
            pick_best_answer("Q?", ["A", "B"], context="Background info.")

        for call_problem in calls:
            self.assertIn("Background info.", call_problem)

    def test_three_way_tie_strict_error_has_all_tied(self) -> None:
        from pureason.reasoning.mcq import AmbiguousAnswerError, pick_best_answer

        reports = [_make_report(65), _make_report(65), _make_report(65)]
        with (
            patch("pureason.reasoning.mcq.verify_chain", side_effect=reports),
            self.assertRaises(AmbiguousAnswerError) as ctx,
        ):
            pick_best_answer("Q?", ["A", "B", "C"], strict=True)
        self.assertEqual(set(ctx.exception.tied_indices), {0, 1, 2})


class TestAmbiguousAnswerError(unittest.TestCase):
    """Unit tests for AmbiguousAnswerError exception class."""

    def test_is_runtime_error(self) -> None:
        from pureason.reasoning.mcq import AmbiguousAnswerError

        self.assertTrue(issubclass(AmbiguousAnswerError, RuntimeError))

    def test_attributes_accessible(self) -> None:
        from pureason.reasoning.mcq import AmbiguousAnswerError

        err = AmbiguousAnswerError([1, 3], 72)
        self.assertEqual(err.tied_indices, [1, 3])
        self.assertEqual(err.ecs, 72)

    def test_message_contains_indices_and_ecs(self) -> None:
        from pureason.reasoning.mcq import AmbiguousAnswerError

        err = AmbiguousAnswerError([0, 2], 85)
        msg = str(err)
        self.assertIn("85", msg)

    def test_single_tied_index(self) -> None:
        from pureason.reasoning.mcq import AmbiguousAnswerError

        err = AmbiguousAnswerError([0], 50)
        self.assertEqual(err.tied_indices, [0])
        self.assertEqual(err.ecs, 50)


if __name__ == "__main__":
    unittest.main()
