"""Result dataclasses returned by the pureason API."""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass
class ScoreBreakdown:
    modality: float = 1.0
    illusion: float = 1.0
    antinomy: float = 1.0
    paralogism: float = 1.0
    game_stability: float = 1.0
    kac: float = 0.5
    entity_coverage: float = 0.5

    @classmethod
    def from_dict(cls, d: dict) -> ScoreBreakdown:
        return cls(
            modality=d.get("modality", 1.0),
            illusion=d.get("illusion", 1.0),
            antinomy=d.get("antinomy", 1.0),
            paralogism=d.get("paralogism", 1.0),
            game_stability=d.get("game_stability", 1.0),
            kac=d.get("kac", 0.5),
            entity_coverage=d.get("entity_coverage", 0.5),
        )


@dataclass
class CalibrationResult:
    """Result of a calibrate() call — ECS + epistemic band."""

    ecs: int
    band: str
    calibrated: bool
    epistemic_mode: str
    flags: list[str]
    safe_version: str
    score_breakdown: ScoreBreakdown

    @property
    def is_safe(self) -> bool:
        return self.band.lower() == "high"

    @property
    def is_flagged(self) -> bool:
        return bool(self.flags) or not self.calibrated

    def __repr__(self) -> str:
        return (
            f"CalibrationResult(ecs={self.ecs}, band={self.band!r}, calibrated={self.calibrated})"
        )


@dataclass
class HallucinationFlag:
    kind: str
    description: str
    severity: str


@dataclass
class AnalysisResult:
    """Result of an analyze() call — full Kantian pipeline + hallucination report."""

    ecs: int
    band: str
    calibrated: bool
    epistemic_mode: str
    flags: list[str]
    safe_version: str
    score_breakdown: ScoreBreakdown

    hallucination_detected: bool = False
    kac_score: float | None = None
    entity_novelty: float | None = None
    hallucination_flags: list[HallucinationFlag] = field(default_factory=list)

    @property
    def is_safe(self) -> bool:
        return self.band.lower() == "high" and not self.hallucination_detected

    def __repr__(self) -> str:
        return (
            f"AnalysisResult(ecs={self.ecs}, band={self.band!r}, "
            f"hallucination={self.hallucination_detected}, kac={self.kac_score})"
        )
