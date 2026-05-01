"""Tests for reasoning data models (StepVerification, EpistemicChainReport)."""

import unittest

from pureason._models import AnalysisResult, CalibrationResult, HallucinationFlag, ScoreBreakdown


class TestScoreBreakdown(unittest.TestCase):
    def test_from_empty_dict_uses_defaults(self) -> None:
        sb = ScoreBreakdown.from_dict({})
        self.assertAlmostEqual(sb.modality, 1.0)
        self.assertAlmostEqual(sb.kac, 0.5)

    def test_from_dict_overrides(self) -> None:
        sb = ScoreBreakdown.from_dict({"modality": 0.8, "kac": 0.2, "antinomy": 0.9})
        self.assertAlmostEqual(sb.modality, 0.8)
        self.assertAlmostEqual(sb.kac, 0.2)
        self.assertAlmostEqual(sb.antinomy, 0.9)


class TestCalibrationResult(unittest.TestCase):
    def _make(self, ecs: int = 75, band: str = "High") -> CalibrationResult:
        return CalibrationResult(
            ecs=ecs,
            band=band,
            calibrated=True,
            epistemic_mode="standard",
            flags=[],
            safe_version="",
            score_breakdown=ScoreBreakdown.from_dict({}),
        )

    def test_basic_fields(self) -> None:
        r = self._make(ecs=80, band="High")
        self.assertEqual(r.ecs, 80)
        self.assertEqual(r.band, "High")
        self.assertTrue(r.calibrated)

    def test_low_ecs(self) -> None:
        r = self._make(ecs=20, band="Low")
        self.assertEqual(r.ecs, 20)
        self.assertEqual(r.band, "Low")


class TestAnalysisResult(unittest.TestCase):
    def _make(self, hallucination_detected: bool = False) -> AnalysisResult:
        return AnalysisResult(
            ecs=60,
            band="Moderate",
            calibrated=True,
            epistemic_mode="standard",
            flags=[],
            safe_version="",
            score_breakdown=ScoreBreakdown.from_dict({}),
            hallucination_detected=hallucination_detected,
            kac_score=None,
            entity_novelty=None,
            hallucination_flags=[],
        )

    def test_no_hallucination(self) -> None:
        r = self._make(hallucination_detected=False)
        self.assertFalse(r.hallucination_detected)
        self.assertEqual(r.hallucination_flags, [])

    def test_hallucination_detected(self) -> None:
        r = self._make(hallucination_detected=True)
        self.assertTrue(r.hallucination_detected)


class TestHallucinationFlag(unittest.TestCase):
    def test_construction(self) -> None:
        flag = HallucinationFlag(kind="KAC", description="Contradiction detected", severity="High")
        self.assertEqual(flag.kind, "KAC")
        self.assertEqual(flag.severity, "High")


if __name__ == "__main__":
    unittest.main()
