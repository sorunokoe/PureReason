"""
LangChain integration for PureReason (S-IV-2).

Drop-in callback handler that validates every LLM output through the
Kantian epistemic pipeline.

Usage:
    from pure_reason.langchain import KantianCallback
    from langchain.chains import LLMChain
    from langchain_openai import ChatOpenAI

    chain = LLMChain(
        llm=ChatOpenAI(),
        callbacks=[KantianCallback(domain="medical", on_high_risk="regulate")]
    )
    # Every LLM output is now epistemically validated automatically.
"""

from __future__ import annotations

from typing import Any

from .exceptions import EpistemicViolationError
from .pipeline import Pipeline


class KantianCallback:
    """
    LangChain BaseCallbackHandler that validates LLM outputs through PureReason.

    Args:
        domain: Domain profile to use ('general', 'medical', 'financial', 'legal', 'scientific').
        on_high_risk: Action when HIGH risk detected:
            - 'warn'     (default): Log a warning, continue.
            - 'regulate': Replace the output with its regulative (corrected) form.
            - 'raise':   Raise EpistemicViolationError.
        min_risk_to_act: Minimum risk level to trigger action ('LOW', 'MEDIUM', 'HIGH').
        api_url: Override REST API URL.
    """

    def __init__(
        self,
        domain: str = "general",
        on_high_risk: str = "warn",
        min_risk_to_act: str = "HIGH",
        api_url: str | None = None,
    ):
        self.pipeline = Pipeline(domain=domain, api_url=api_url)
        self.on_high_risk = on_high_risk
        self.min_risk_to_act = min_risk_to_act
        self._risk_order = {"SAFE": 0, "LOW": 1, "MEDIUM": 2, "HIGH": 3, "CRITICAL": 4}

    # ── LangChain callback protocol ──────────────────────────────────────────

    def on_llm_end(self, response: Any, **kwargs: Any) -> None:
        """Called when the LLM finishes generating. Validates the output."""
        try:
            generations = getattr(response, "generations", [[]])
            for gen_list in generations:
                for gen in gen_list:
                    text = getattr(gen, "text", None) or str(gen)
                    self._handle_output(gen, text)
        except Exception:
            pass  # Never break the user's application

    def _handle_output(self, gen: Any, text: str) -> None:
        if not text or not text.strip():
            return

        result = self.pipeline.validate(text)
        risk = result.get("risk_level", "SAFE")

        if self._risk_order.get(risk, 0) < self._risk_order.get(self.min_risk_to_act, 3):
            return  # Below threshold — no action

        if self.on_high_risk == "raise":
            full_report = self.pipeline.analyze(text)
            raise EpistemicViolationError(full_report)

        elif self.on_high_risk == "regulate":
            regulated = self.pipeline.regulate(text)
            if hasattr(gen, "text"):
                gen.text = regulated
            return

        else:  # warn (default)
            import warnings

            issues = []
            if result.get("has_illusions"):
                issues.append("transcendental illusion")
            if result.get("has_contradictions"):
                issues.append("contradiction")
            if result.get("has_paralogisms"):
                issues.append("paralogism")
            warnings.warn(
                f"PureReason: {risk} risk detected in LLM output. "
                f"Issues: {', '.join(issues) or 'see full report'}. "
                f"Summary: {result.get('summary', '')}",
                stacklevel=3,
            )

    # Satisfy LangChain's callback interface (no-ops for other events)
    def on_llm_start(self, *args: Any, **kwargs: Any) -> None:
        pass

    def on_llm_error(self, *args: Any, **kwargs: Any) -> None:
        pass

    def on_chain_start(self, *args: Any, **kwargs: Any) -> None:
        pass

    def on_chain_end(self, *args: Any, **kwargs: Any) -> None:
        pass

    def on_chain_error(self, *args: Any, **kwargs: Any) -> None:
        pass

    def on_tool_start(self, *args: Any, **kwargs: Any) -> None:
        pass

    def on_tool_end(self, *args: Any, **kwargs: Any) -> None:
        pass

    def on_tool_error(self, *args: Any, **kwargs: Any) -> None:
        pass
