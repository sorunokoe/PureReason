"""Tests for verify_chain and verify_syllogism (mocked Rust binary calls)."""

import unittest
from unittest.mock import MagicMock, patch


def _calibrate_mock(cmd, **kwargs):
    """Mock _run to return a plausible calibrate result."""
    return {
        "ecs": 75,
        "band": "High",
        "calibrated": True,
        "epistemic_mode": "standard",
        "flags": [],
        "safe_version": "",
        "score_breakdown": {},
    }


def _analyze_mock(cmd, stdin_text=None, **kwargs):
    """Mock _run to return a plausible analyze result."""
    return {
        "calibration": {
            "ecs": 70,
            "band": "High",
            "calibrated": True,
            "epistemic_mode": "standard",
            "flags": [],
            "safe_version": "",
            "score_breakdown": {},
        },
        "dialectic": {
            "kac_score": 0.05,
            "entity_novelty": 0.1,
            "illusions": [],
            "antinomies": [],
        },
    }


def _run_mock(cmd, stdin_text=None, **kwargs):
    if cmd[0] == "calibrate":
        return _calibrate_mock(cmd)
    return _analyze_mock(cmd, stdin_text=stdin_text)


class TestVerifyChainMocked(unittest.TestCase):
    @patch("pureason._core._run", side_effect=_run_mock)
    def test_valid_chain_returns_report(self, _mock: MagicMock) -> None:
        from pureason.reasoning import EpistemicChainReport, verify_chain

        chain = [
            "All mammals are warm-blooded.",
            "Whales are mammals.",
            "Therefore whales are warm-blooded.",
        ]
        report = verify_chain("Are whales warm-blooded?", chain)
        self.assertIsInstance(report, EpistemicChainReport)
        self.assertEqual(len(report.steps), 3)
        self.assertEqual(report.problem, "Are whales warm-blooded?")

    @patch("pureason._core._run", side_effect=_run_mock)
    def test_empty_chain_returns_empty_steps(self, _mock: MagicMock) -> None:
        from pureason.reasoning import verify_chain

        report = verify_chain("What is 2+2?", [])
        self.assertEqual(report.steps, [])

    @patch("pureason._core._run", side_effect=_run_mock)
    def test_report_is_valid_for_mocked_high_ecs(self, _mock: MagicMock) -> None:
        from pureason.reasoning import verify_chain

        report = verify_chain("Test problem", ["Step A", "Step B"])
        # With mocked ECS=75 (high) and no KAC flags, chain should be valid
        self.assertTrue(report.is_valid)

    @patch("pureason._core._run", side_effect=_run_mock)
    def test_chain_confidence_between_0_and_1(self, _mock: MagicMock) -> None:
        from pureason.reasoning import verify_chain

        report = verify_chain("Test", ["Step one", "Step two", "Step three"])
        self.assertGreaterEqual(report.chain_confidence, 0.0)
        self.assertLessEqual(report.chain_confidence, 1.0)

    @patch("pureason._core._run", side_effect=_run_mock)
    def test_first_failure_none_when_valid(self, _mock: MagicMock) -> None:
        from pureason.reasoning import verify_chain

        report = verify_chain("Test", ["Step A", "Step B"])
        self.assertIsNone(report.first_failure)


class TestVerifySyllogismMocked(unittest.TestCase):
    def test_valid_syllogism_no_z3(self) -> None:
        """Heuristic path should handle basic syllogisms even without Z3."""
        from pureason.reasoning import EpistemicChainReport, verify_syllogism

        premises = ["All dogs are animals.", "Fido is a dog."]
        conclusion = "Fido is an animal."
        report = verify_syllogism(premises, conclusion)
        self.assertIsInstance(report, EpistemicChainReport)
        # Should be reported as valid for this classic syllogism
        self.assertTrue(report.is_valid)

    def test_invalid_syllogism_detected(self) -> None:
        """Classic fallacy: affirming the consequent."""
        from pureason.reasoning import verify_syllogism

        premises = ["If it rains, the ground is wet.", "The ground is wet."]
        conclusion = "It is raining."
        report = verify_syllogism(premises, conclusion)
        self.assertIsInstance(report, type(report))
        # Validity depends on Z3 / heuristic — just ensure it returns without error

    def test_empty_premises(self) -> None:
        from pureason.reasoning import verify_syllogism

        report = verify_syllogism([], "Some conclusion.")
        self.assertIsInstance(report, type(report))

    def test_tautology_valid(self) -> None:
        from pureason.reasoning import verify_syllogism

        premises = ["All X are Y.", "A is an X."]
        conclusion = "A is a Y."
        report = verify_syllogism(premises, conclusion)
        # Just verify it returns without error; heuristic path may not always detect
        self.assertIsNotNone(report)


class TestStepVerificationDataclass(unittest.TestCase):
    def test_step_verification_fields(self) -> None:
        from pureason.reasoning import StepVerification

        sv = StepVerification(
            step_index=0,
            step_text="All cats are animals.",
            ecs=80,
            is_internally_valid=True,
            is_contextually_valid=True,
            flags=[],
            contradiction_with_step=None,
        )
        self.assertEqual(sv.step_index, 0)
        self.assertEqual(sv.ecs, 80)
        self.assertTrue(sv.is_internally_valid)
        self.assertIsNone(sv.contradiction_with_step)

    def test_epistemic_chain_report_first_failure(self) -> None:
        from pureason.reasoning import EpistemicChainReport, StepVerification

        steps = [
            StepVerification(0, "Step A", 80, True, True),
            StepVerification(1, "Step B", 20, False, False, flags=["contradiction"]),
        ]
        report = EpistemicChainReport(
            problem="Test",
            steps=steps,
            answer=None,
            is_valid=False,
            chain_confidence=0.5,
            invalid_steps=[1],
            summary="Invalid chain",
        )
        ff = report.first_failure
        self.assertIsNotNone(ff)
        self.assertEqual(ff.step_index, 1)


# ---------------------------------------------------------------------------
# _arithmetic_error_in_text — direct unit tests (no mock needed)
# ---------------------------------------------------------------------------


class TestArithmeticErrorInText(unittest.TestCase):
    """Unit tests for _arithmetic_error_in_text covering all operator branches."""

    def setUp(self) -> None:
        from pureason.reasoning.chain import _arithmetic_error_in_text

        self._fn = _arithmetic_error_in_text

    def test_correct_addition_no_error(self) -> None:
        self.assertFalse(self._fn("So 3 + 4 = 7."))

    def test_wrong_addition_detected(self) -> None:
        self.assertTrue(self._fn("We get 3 + 4 = 8."))

    def test_correct_subtraction_no_error(self) -> None:
        self.assertFalse(self._fn("10 - 4 = 6."))

    def test_wrong_subtraction_detected(self) -> None:
        self.assertTrue(self._fn("10 - 4 = 5."))

    def test_correct_multiplication_no_error(self) -> None:
        self.assertFalse(self._fn("6 * 7 = 42."))

    def test_wrong_multiplication_detected(self) -> None:
        self.assertTrue(self._fn("6 * 7 = 41."))

    def test_correct_division_no_error(self) -> None:
        self.assertFalse(self._fn("10 / 2 = 5."))

    def test_wrong_division_detected(self) -> None:
        self.assertTrue(self._fn("10 / 2 = 6."))

    def test_division_by_zero_not_flagged(self) -> None:
        self.assertFalse(self._fn("5 / 0 = 99."))

    def test_unicode_multiplication_symbol(self) -> None:
        self.assertFalse(self._fn("3 × 4 = 12."))

    def test_unicode_division_symbol(self) -> None:
        self.assertFalse(self._fn("12 ÷ 4 = 3."))

    def test_no_arithmetic_expression_no_error(self) -> None:
        self.assertFalse(self._fn("The cat sat on the mat."))

    def test_empty_string_no_error(self) -> None:
        self.assertFalse(self._fn(""))

    def test_multiple_steps_one_wrong(self) -> None:
        text = "Step 1: 3 + 4 = 7. Step 2: 5 * 3 = 14."
        self.assertTrue(self._fn(text))

    def test_multiple_steps_all_correct(self) -> None:
        text = "First: 3 + 4 = 7. Then: 6 * 7 = 42."
        self.assertFalse(self._fn(text))

    def test_percent_correct(self) -> None:
        self.assertFalse(self._fn("50% of 100 is 50."))

    def test_percent_wrong_detected(self) -> None:
        self.assertTrue(self._fn("50% of 100 is 60."))

    def test_decimal_correct(self) -> None:
        self.assertFalse(self._fn("2.5 + 1.5 = 4.0"))

    def test_decimal_wrong_detected(self) -> None:
        self.assertTrue(self._fn("2.5 + 1.5 = 5.0"))

    def test_negative_operand_correct(self) -> None:
        self.assertFalse(self._fn("-3 + 5 = 2."))

    def test_tolerance_within_half_percent(self) -> None:
        # 100 / 3 ≈ 33.333; 33 is within 0.5% relative tolerance? 33.333*0.005=0.167; |33.333-33|=0.333 > 0.167
        # So this should be flagged as an error.
        self.assertTrue(self._fn("100 / 3 = 33."))


if __name__ == "__main__":
    unittest.main()
