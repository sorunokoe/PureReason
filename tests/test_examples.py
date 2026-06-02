"""Tests for example use cases — validates that all examples produce expected results.

These tests exercise the Python reasoning layer without requiring the Rust binary
by mocking _core._run where needed.  Pure-Python functionality (arithmetic,
repair, models) is tested directly.
"""

import sys
import unittest
from unittest.mock import MagicMock, patch

sys.path.insert(0, ".")


# ---------------------------------------------------------------------------
# 1. ReasoningGuard
# ---------------------------------------------------------------------------


class TestGuardUseCases(unittest.TestCase):
    """Tests covering guard_verification.py use cases."""

    def test_guard_verified_provenance(self) -> None:
        """High ECS → provenance='verified'."""
        from pureason.guard import ReasoningGuard

        with patch("pureason.reasoning.chain._run") as mock_run:
            mock_run.return_value = {"ecs": 80, "flags": []}
            guard = ReasoningGuard(threshold=60)
            result = guard.verify("Water boils at 100 degrees Celsius.")
            self.assertEqual(result.provenance, "verified")
            self.assertGreaterEqual(result.ecs, 60)

    def test_guard_flagged_provenance(self) -> None:
        """Low ECS with no repairable content → provenance='flagged'."""
        from pureason.guard import ReasoningGuard

        with patch("pureason.reasoning.chain._run") as mock_run:
            mock_run.return_value = {"ecs": 20, "flags": ["CERTAINTY_OVERREACH"]}
            guard = ReasoningGuard(threshold=60, repair=True)
            result = guard.verify("This is definitely absolutely true.")
            self.assertEqual(result.provenance, "flagged")

    def test_guard_repaired_provenance(self) -> None:
        """Low ECS with arithmetic error → provenance='repaired'."""
        from pureason.guard import ReasoningGuard

        with patch("pureason.reasoning.chain._run") as mock_run:
            mock_run.return_value = {"ecs": 30, "flags": []}
            guard = ReasoningGuard(threshold=60, repair=True)
            result = guard.verify("3 + 4 = 8 so the total is wrong.")
            self.assertEqual(result.provenance, "repaired")
            self.assertTrue(result.repaired)
            self.assertIn("7", result.text)

    def test_guard_threshold_affects_outcome(self) -> None:
        """Same text should be 'verified' at low threshold, 'flagged' at high."""
        from pureason.guard import ReasoningGuard

        with patch("pureason.reasoning.chain._run") as mock_run:
            mock_run.return_value = {"ecs": 55, "flags": []}

            low_guard = ReasoningGuard(threshold=40)
            high_guard = ReasoningGuard(threshold=60)

            r_low = low_guard.verify("Some text.")
            r_high = high_guard.verify("Some text.")

            self.assertEqual(r_low.provenance, "verified")
            self.assertEqual(r_high.provenance, "flagged")

    def test_guard_repair_disabled(self) -> None:
        """When repair=False, arithmetic errors are not corrected."""
        from pureason.guard import ReasoningGuard

        with patch("pureason.reasoning.chain._run") as mock_run:
            mock_run.return_value = {"ecs": 30, "flags": []}
            guard = ReasoningGuard(threshold=60, repair=False)
            result = guard.verify("3 + 4 = 8")
            self.assertFalse(result.repaired)
            self.assertEqual(result.provenance, "flagged")


# ---------------------------------------------------------------------------
# 2. Chain-of-Thought Verification
# ---------------------------------------------------------------------------


class TestChainOfThoughtUseCases(unittest.TestCase):
    """Tests covering chain_of_thought.py use cases."""

    @patch("pureason.reasoning.chain._run")
    def test_valid_chain_all_pass(self, mock_run: MagicMock) -> None:
        """A correct chain should report is_valid=True."""
        from pureason.reasoning import verify_chain

        mock_run.return_value = {"ecs": 75, "flags": []}
        report = verify_chain(
            "What is 50 - 12?",
            [
                "The store starts with 50 apples.",
                "A customer buys 12 apples.",
                "Remaining = 50 - 12 = 38.",
                "Therefore, the answer is 38.",
            ],
        )
        self.assertTrue(report.is_valid)
        self.assertEqual(len(report.invalid_steps), 0)
        self.assertGreater(report.chain_confidence, 0)

    @patch("pureason.reasoning.chain._run")
    def test_chain_with_arithmetic_error(self, mock_run: MagicMock) -> None:
        """A chain containing '15 + 27 = 43' should flag that step."""
        from pureason.reasoning import verify_chain

        mock_run.return_value = {"ecs": 60, "flags": []}
        report = verify_chain(
            "What is 15 + 27?",
            [
                "We add the numbers.",
                "15 + 27 = 43.",
            ],
        )
        # The step "15 + 27 = 43" is at index 1 (second step).
        # 15 + 27 = 42, not 43, so it should be flagged.
        arith_flagged = any("ARITHMETIC_ERROR" in sv.flags for sv in report.steps)
        self.assertTrue(arith_flagged, "Arithmetic error should be flagged")

    @patch("pureason.reasoning.chain._run")
    def test_empty_chain(self, mock_run: MagicMock) -> None:
        """Empty step list → is_valid=False, confidence=0."""
        from pureason.reasoning import verify_chain

        report = verify_chain("Any?", [])
        self.assertFalse(report.is_valid)
        self.assertEqual(report.chain_confidence, 0.0)
        self.assertIsNone(report.answer)

    @patch("pureason.reasoning.chain._run")
    def test_single_step_chain(self, mock_run: MagicMock) -> None:
        """Single-step chain should still produce a valid report."""
        from pureason.reasoning import verify_chain

        mock_run.return_value = {"ecs": 70, "flags": []}
        report = verify_chain("What is 2 + 2?", ["2 + 2 = 4."])
        self.assertEqual(len(report.steps), 1)
        self.assertEqual(report.answer, "2 + 2 = 4.")


# ---------------------------------------------------------------------------
# 3. Arithmetic — Pure Python, no mocking needed
# ---------------------------------------------------------------------------


class TestArithmeticUseCases(unittest.TestCase):
    """Tests covering arithmetic_solver.py use cases."""

    def test_safe_eval_basic_operations(self) -> None:
        from pureason.reasoning.arithmetic import _safe_eval

        self.assertAlmostEqual(_safe_eval("2 + 3"), 5.0)
        self.assertAlmostEqual(_safe_eval("10 - 4"), 6.0)
        self.assertAlmostEqual(_safe_eval("6 * 7"), 42.0)
        self.assertAlmostEqual(_safe_eval("10 / 4"), 2.5)

    def test_safe_eval_rejects_dangerous_input(self) -> None:
        from pureason.reasoning.arithmetic import _safe_eval

        self.assertIsNone(_safe_eval("import os"))
        self.assertIsNone(_safe_eval("__import__('os')"))
        self.assertIsNone(_safe_eval(""))

    def test_safe_eval_division_by_zero(self) -> None:
        from pureason.reasoning.arithmetic import _safe_eval

        self.assertIsNone(_safe_eval("5 / 0"))

    def test_extract_numbers_digits(self) -> None:
        from pureason.reasoning.arithmetic import _extract_numbers

        nums = _extract_numbers("There are 3 apples and 10 bananas.")
        self.assertIn(3.0, nums)
        self.assertIn(10.0, nums)

    def test_extract_numbers_decimals(self) -> None:
        from pureason.reasoning.arithmetic import _extract_numbers

        nums = _extract_numbers("The price is 3.14 dollars.")
        self.assertIn(3.14, nums)

    def test_extract_numbers_negative(self) -> None:
        from pureason.reasoning.arithmetic import _extract_numbers

        nums = _extract_numbers("Temperature is -5 degrees.")
        self.assertIn(-5.0, nums)

    def test_extract_numbers_comma_separated(self) -> None:
        from pureason.reasoning.arithmetic import _extract_numbers

        nums = _extract_numbers("The factory produced 1,000 units.")
        self.assertIn(1000.0, nums)

    def test_detect_operation_addition(self) -> None:
        from pureason.reasoning.arithmetic import _detect_operation

        op = _detect_operation("How many total items if we add 3 more?")
        self.assertEqual(op, "+")

    def test_detect_operation_subtraction(self) -> None:
        from pureason.reasoning.arithmetic import _detect_operation

        op = _detect_operation("How many are left after removing 5?")
        self.assertEqual(op, "-")

    def test_detect_operation_division(self) -> None:
        from pureason.reasoning.arithmetic import _detect_operation

        op = _detect_operation("What is the average speed?")
        self.assertEqual(op, "/")


# ---------------------------------------------------------------------------
# 4. Repair — Pure Python, no mocking needed
# ---------------------------------------------------------------------------


class TestRepairUseCases(unittest.TestCase):
    """Tests covering arithmetic_repair.py use cases."""

    def test_correct_expression_not_repaired(self) -> None:
        from pureason.reasoning.repair import _repair_arithmetic_in_step

        result = _repair_arithmetic_in_step("3 + 4 = 7 apples.")
        self.assertNotIn("[repaired]", result)

    def test_wrong_addition_repaired(self) -> None:
        from pureason.reasoning.repair import _repair_arithmetic_in_step

        result = _repair_arithmetic_in_step("3 + 4 = 8 apples.")
        self.assertIn("[repaired]", result)
        self.assertIn("7", result)

    def test_wrong_multiplication_repaired(self) -> None:
        from pureason.reasoning.repair import _repair_arithmetic_in_step

        result = _repair_arithmetic_in_step("6 * 7 = 41")
        self.assertIn("[repaired]", result)
        self.assertIn("42", result)

    def test_wrong_subtraction_repaired(self) -> None:
        from pureason.reasoning.repair import _repair_arithmetic_in_step

        result = _repair_arithmetic_in_step("100 - 37 = 64")
        self.assertIn("[repaired]", result)
        self.assertIn("63", result)

    def test_extract_numeric_answer(self) -> None:
        from pureason.reasoning.repair import _extract_numeric_answer

        self.assertEqual(_extract_numeric_answer("The answer is 42."), 42.0)
        self.assertIsNone(_extract_numeric_answer("No number here at all."))

    def test_extract_letter_answer(self) -> None:
        from pureason.reasoning.repair import _extract_letter_answer

        self.assertEqual(_extract_letter_answer("Therefore the answer is A."), "A")
        self.assertEqual(_extract_letter_answer("The best answer is **B**."), "B")
        self.assertIsNone(_extract_letter_answer("No clear MCQ answer here."))

    def test_majority_vote_numeric(self) -> None:
        from pureason.reasoning.repair import _majority_vote

        self.assertEqual(_majority_vote([42.0, 42.0, 41.0, 42.0]), 42.0)
        self.assertIsNone(_majority_vote([]))

    def test_majority_vote_letters(self) -> None:
        from pureason.reasoning.repair import _majority_vote_letters

        self.assertEqual(_majority_vote_letters(["A", "B", "A", "A"]), "A")
        self.assertEqual(_majority_vote_letters([None, "B", None, "B"]), "B")
        self.assertIsNone(_majority_vote_letters([]))


# ---------------------------------------------------------------------------
# 5. MCQ Picker
# ---------------------------------------------------------------------------


class TestMCQUseCases(unittest.TestCase):
    """Tests covering mcq_picker.py use cases."""

    @patch("pureason.reasoning.chain._run")
    def test_picks_an_index(self, mock_run: MagicMock) -> None:
        """pick_best_answer should return a valid index."""
        from pureason.reasoning import pick_best_answer

        # Return different ECS for each choice to make one clearly best
        call_count = 0

        def side_effect(*args, **kwargs):
            nonlocal call_count
            call_count += 1
            ecs_values = [80, 50, 60, 40]
            idx = min(call_count - 1, len(ecs_values) - 1)
            return {"ecs": ecs_values[idx], "flags": []}

        mock_run.side_effect = side_effect

        choices = ["Paris", "Berlin", "Madrid", "Rome"]
        best_idx, _report = pick_best_answer("Capital of France?", choices)
        self.assertIn(best_idx, range(len(choices)))

    def test_empty_choices_raises(self) -> None:
        from pureason.reasoning import pick_best_answer

        with self.assertRaises(ValueError):
            pick_best_answer("Question?", [])

    @patch("pureason.reasoning.chain._run")
    def test_strict_mode_raises_on_tie(self, mock_run: MagicMock) -> None:
        """When all choices get the same ECS, strict mode raises AmbiguousAnswerError."""
        from pureason.reasoning import pick_best_answer
        from pureason.reasoning.mcq import AmbiguousAnswerError

        mock_run.return_value = {"ecs": 50, "flags": []}
        with self.assertRaises(AmbiguousAnswerError):
            pick_best_answer("Pick one.", ["A", "B"], strict=True)

    @patch("pureason.reasoning.chain._run")
    def test_lenient_mode_flags_tie(self, mock_run: MagicMock) -> None:
        """Lenient mode returns first index and adds MCQ_AMBIGUOUS_ECS_TIE flag."""
        from pureason.reasoning import pick_best_answer

        mock_run.return_value = {"ecs": 50, "flags": []}
        best_idx, report = pick_best_answer("Pick one.", ["A", "B"], strict=False)
        self.assertEqual(best_idx, 0)
        if report.steps:
            self.assertIn("MCQ_AMBIGUOUS_ECS_TIE", report.steps[0].flags)


# ---------------------------------------------------------------------------
# 6. Syllogism Verification
# ---------------------------------------------------------------------------


class TestSyllogismUseCases(unittest.TestCase):
    """Tests covering syllogism_verification.py use cases."""

    @patch("pureason.reasoning.chain._run")
    def test_valid_syllogism(self, mock_run: MagicMock) -> None:
        """Classic valid syllogism should be detected as valid."""
        from pureason.reasoning import verify_syllogism

        mock_run.return_value = {"ecs": 75, "flags": []}
        report = verify_syllogism(
            premises=["All mammals are warm-blooded.", "Whales are mammals."],
            conclusion="Whales are warm-blooded.",
        )
        self.assertTrue(report.is_valid)

    def test_invalid_syllogism(self) -> None:
        """Invalid syllogism — conclusion does not follow from premises.

        We mock the classifier, Z3, and KAC layers to isolate the heuristic
        fallacy check which detects that no universal premises support
        the universal conclusion.
        """
        from pureason.reasoning import verify_syllogism

        with (
            patch("pureason.reasoning.syllogism._classifier_check", return_value=None),
            patch("pureason.reasoning.syllogism._z3_entailment_check", return_value=None),
            patch("pureason.reasoning.syllogism._kac_step_vs_context", return_value=(False, [])),
        ):
            report = verify_syllogism(
                premises=["All dogs are animals.", "All cats are animals."],
                conclusion="All dogs are cats.",
            )
            self.assertFalse(report.is_valid)


# ---------------------------------------------------------------------------
# 7. Models — dataclass structure
# ---------------------------------------------------------------------------


class TestModelUseCases(unittest.TestCase):
    """Test model dataclass structure used across all examples."""

    def test_step_verification_fields(self) -> None:
        from pureason.reasoning.models import StepVerification

        sv = StepVerification(
            step_index=0,
            step_text="test",
            ecs=75,
            is_internally_valid=True,
            is_contextually_valid=True,
            flags=["TEST_FLAG"],
        )
        self.assertEqual(sv.step_index, 0)
        self.assertEqual(sv.ecs, 75)
        self.assertTrue(sv.is_internally_valid)
        self.assertIsNone(sv.contradiction_with_step)

    def test_chain_report_first_failure(self) -> None:
        from pureason.reasoning.models import EpistemicChainReport, StepVerification

        sv_ok = StepVerification(0, "ok", 80, True, True)
        sv_bad = StepVerification(1, "bad", 20, False, True, flags=["ERROR"])
        report = EpistemicChainReport(
            problem="test",
            steps=[sv_ok, sv_bad],
            answer="bad",
            is_valid=False,
            chain_confidence=0.3,
            invalid_steps=[1],
            summary="Step 2 failed.",
        )
        self.assertIsNotNone(report.first_failure)
        self.assertEqual(report.first_failure.step_index, 1)

    def test_chain_report_no_failure(self) -> None:
        from pureason.reasoning.models import EpistemicChainReport, StepVerification

        sv_ok = StepVerification(0, "ok", 80, True, True)
        report = EpistemicChainReport(
            problem="test",
            steps=[sv_ok],
            answer="ok",
            is_valid=True,
            chain_confidence=0.8,
            invalid_steps=[],
            summary="All passed.",
        )
        self.assertIsNone(report.first_failure)


if __name__ == "__main__":
    unittest.main()
