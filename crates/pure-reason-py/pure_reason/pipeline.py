"""
Core Pipeline wrapper for PureReason.

Tries to import the native PyO3 extension first; falls back to REST API.
"""

from __future__ import annotations

import json
import os


def _get_native():
    """Try to import the native PyO3 extension module."""
    try:
        import pure_reason as _native

        # The compiled module has the same name — check it has the right class
        if hasattr(_native, "PureReason"):
            return _native.PureReason()
    except ImportError:
        pass
    return None


def _get_api_url() -> str | None:
    """Return REST API URL if configured."""
    return os.environ.get("PURE_REASON_API_URL")


class Pipeline:
    """
    The PureReason Kantian pipeline.

    Uses the native PyO3 extension when available (fastest, zero network).
    Falls back to the REST API if PURE_REASON_API_URL is set.

    Args:
        domain: Optional domain profile name ('medical', 'financial', 'legal', 'scientific').
        api_url: Override the REST API URL (default: PURE_REASON_API_URL env var).
    """

    def __init__(self, domain: str = "general", api_url: str | None = None):
        self.domain = domain
        self._native = _get_native()
        self._api_url = api_url or _get_api_url()

    def analyze(self, text: str) -> dict:
        """Run the full Kantian pipeline analysis.

        Args:
            text: The text to analyze.

        Returns:
            dict: Full PipelineReport with verdict, dialectic, understanding, etc.
        """
        if self._native:
            return self._native.analyze(text)
        return self._api_call("analyze", text)

    def validate(self, text: str) -> dict:
        """Quick validation: risk level + issue flags.

        Args:
            text: The text to validate.

        Returns:
            dict: {risk_level, has_illusions, has_contradictions, has_paralogisms, summary}
        """
        if self._native:
            return self._native.validate(text)
        return self._api_call("validate", text)

    def certify(self, text: str) -> dict:
        """Generate a content-addressed ValidationCertificate.

        Args:
            text: The text to certify.

        Returns:
            dict: Certificate with content_hash, risk_level, issued_at, issues.
        """
        if self._native:
            return self._native.certify(text)
        return self._api_call("certify", text)

    def regulate(self, text: str) -> str:
        """Apply regulative transformation to epistemic overreach.

        Args:
            text: The text to transform.

        Returns:
            str: The regulated text.
        """
        if self._native:
            return self._native.regulate(text)
        result = self._api_call("regulate", text)
        return result.get("regulated_text", text)

    def risk(self, text: str) -> str:
        """Return just the risk level string ('SAFE', 'LOW', 'MEDIUM', 'HIGH').

        Args:
            text: The text to assess.

        Returns:
            str: Risk level.
        """
        result = self.validate(text)
        return result.get("risk_level", "SAFE")

    def _api_call(self, endpoint: str, text: str) -> dict:
        if not self._api_url:
            raise RuntimeError(
                "No native PureReason extension found and PURE_REASON_API_URL is not set. "
                "Install: pip install pure-reason (with native extension) "
                "or set PURE_REASON_API_URL=http://localhost:8080"
            )
        try:
            import urllib.request

            url = f"{self._api_url.rstrip('/')}/api/v1/{endpoint}"
            data = json.dumps({"text": text}).encode()
            req = urllib.request.Request(
                url, data=data, headers={"Content-Type": "application/json"}
            )
            with urllib.request.urlopen(req, timeout=30) as resp:
                return json.loads(resp.read().decode())
        except Exception as e:
            raise RuntimeError(f"PureReason API call failed: {e}") from e


# ─── Convenience functions ────────────────────────────────────────────────────

_default_pipeline: Pipeline | None = None


def _pipeline() -> Pipeline:
    global _default_pipeline
    if _default_pipeline is None:
        _default_pipeline = Pipeline()
    return _default_pipeline


def validate(text: str, domain: str = "general") -> dict:
    """Quick validation. Returns {risk_level, has_illusions, has_contradictions, summary}."""
    return Pipeline(domain=domain).validate(text)


def regulate(text: str) -> str:
    """Apply regulative transformation and return the corrected text."""
    return _pipeline().regulate(text)


def certify(text: str) -> dict:
    """Generate a content-addressed certificate for the text."""
    return _pipeline().certify(text)
