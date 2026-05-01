"""Exceptions raised by PureReason framework integrations."""


class PureReasonError(Exception):
    """Base exception for all PureReason errors."""

    pass


class EpistemicViolationError(PureReasonError):
    """Raised when an LLM output contains high-risk epistemic violations.

    Attributes:
        report: The full PipelineReport dict from PureReason analysis.
        risk_level: The risk level string ('MEDIUM', 'HIGH').
        issues: List of issue type strings.
    """

    def __init__(self, report: dict):
        self.report = report
        self.risk_level = report.get("verdict", {}).get("risk", "HIGH")
        self.issues = [
            f"{i.get('kind', 'Unknown')}" for i in report.get("dialectic", {}).get("illusions", [])
        ]
        super().__init__(
            f"Epistemic violation detected (risk={self.risk_level}): {', '.join(self.issues) or 'see report'}"
        )
