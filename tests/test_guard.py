"""Tests for guard.py (ReasoningGuard) — mocked to avoid Rust binary."""

import unittest
from unittest.mock import MagicMock, patch


class TestReasoningGuardStructure(unittest.TestCase):
    """Test the ReasoningGuard class structure and interface without binary."""

    def test_guard_importable(self) -> None:
        from pureason.guard import ReasoningGuard

        self.assertTrue(callable(ReasoningGuard))

    def test_guard_instantiation(self) -> None:
        from pureason.guard import ReasoningGuard

        guard = ReasoningGuard()
        self.assertIsNotNone(guard)

    def test_guard_has_verify_method(self) -> None:
        from pureason.guard import ReasoningGuard

        guard = ReasoningGuard()
        self.assertTrue(hasattr(guard, "verify"))
        self.assertTrue(callable(guard.verify))


class TestReasoningGuardVerifyMocked(unittest.TestCase):
    """Test verify() logic using mocked subprocess calls."""

    @patch("pureason._core._run")
    def test_verify_returns_result(self, mock_run: MagicMock) -> None:
        from pureason.guard import ReasoningGuard

        mock_run.return_value = {
            "ecs": 80,
            "band": "High",
            "calibrated": True,
            "epistemic_mode": "standard",
            "flags": [],
            "safe_version": "safe text",
            "score_breakdown": {},
        }
        guard = ReasoningGuard()
        result = guard.verify("The Earth orbits the Sun.")
        self.assertIsNotNone(result)

    @patch("pureason._core._run")
    def test_verify_returns_provenance_string(self, mock_run: MagicMock) -> None:
        from pureason.guard import ReasoningGuard, VerificationResult

        mock_run.return_value = {
            "ecs": 30,
            "band": "Low",
            "calibrated": False,
            "epistemic_mode": "standard",
            "flags": [],
            "safe_version": "",
            "score_breakdown": {},
        }
        guard = ReasoningGuard(threshold=60)
        result = guard.verify("Some text.")
        self.assertIsInstance(result, VerificationResult)
        self.assertIn(result.provenance, ("verified", "repaired", "flagged"))


if __name__ == "__main__":
    unittest.main()
