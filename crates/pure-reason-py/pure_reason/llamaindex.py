"""
LlamaIndex integration for PureReason (S-IV-2).

Provides a NodePostprocessor that filters retrieved nodes by epistemic risk,
and a QueryTransform that validates query results.

Usage:
    from pure_reason.llamaindex import KantianPostprocessor
    from llama_index.core import VectorStoreIndex

    index = VectorStoreIndex.from_documents(docs)
    query_engine = index.as_query_engine(
        node_postprocessors=[KantianPostprocessor(domain="legal", max_risk="MEDIUM")]
    )
"""

from __future__ import annotations

from typing import Any

from .exceptions import EpistemicViolationError
from .pipeline import Pipeline


class KantianPostprocessor:
    """
    LlamaIndex NodePostprocessor that filters nodes by epistemic risk.

    Nodes with risk above `max_risk` are excluded from the context sent to the LLM.
    This prevents high-risk retrieved content from contaminating LLM generation.

    Args:
        domain: Domain profile ('general', 'medical', 'financial', 'legal', 'scientific').
        max_risk: Maximum acceptable risk level ('LOW', 'MEDIUM', 'HIGH').
        on_high_risk: 'filter' (default, remove node) or 'regulate' (replace text).
        api_url: Override REST API URL.
    """

    def __init__(
        self,
        domain: str = "general",
        max_risk: str = "MEDIUM",
        on_high_risk: str = "filter",
        api_url: str | None = None,
    ):
        self.pipeline = Pipeline(domain=domain, api_url=api_url)
        self.max_risk = max_risk
        self.on_high_risk = on_high_risk
        self._risk_order = {"SAFE": 0, "LOW": 1, "MEDIUM": 2, "HIGH": 3, "CRITICAL": 4}

    def postprocess_nodes(self, nodes: list[Any], query_bundle: Any = None) -> list[Any]:
        """Filter or regulate nodes based on epistemic risk."""
        result = []
        for node in nodes:
            text = self._get_text(node)
            if not text:
                result.append(node)
                continue

            risk = self.pipeline.risk(text)
            if self._risk_order.get(risk, 0) <= self._risk_order.get(self.max_risk, 2):
                result.append(node)
            elif self.on_high_risk == "regulate":
                regulated = self.pipeline.regulate(text)
                node = self._set_text(node, regulated)
                result.append(node)
            # else: filter — node is dropped

        return result

    def _postprocess_nodes(self, nodes: list[Any], query_bundle: Any = None) -> list[Any]:
        """LlamaIndex v0.10+ interface."""
        return self.postprocess_nodes(nodes, query_bundle)

    @staticmethod
    def _get_text(node: Any) -> str:
        if hasattr(node, "get_content"):
            return node.get_content()
        if hasattr(node, "text"):
            return node.text
        return str(node)

    @staticmethod
    def _set_text(node: Any, text: str) -> Any:
        if hasattr(node, "node") and hasattr(node.node, "text"):
            node.node.text = text
        elif hasattr(node, "text"):
            node.text = text
        return node


class KantianResponseValidator:
    """
    Validates the final LlamaIndex query response through PureReason.

    Usage:
        response = query_engine.query("What is the prognosis?")
        validator = KantianResponseValidator(domain="medical")
        safe_response = validator.validate(response)
    """

    def __init__(
        self, domain: str = "general", on_high_risk: str = "regulate", api_url: str | None = None
    ):
        self.pipeline = Pipeline(domain=domain, api_url=api_url)
        self.on_high_risk = on_high_risk

    def validate(self, response: Any) -> Any:
        text = str(response)
        risk = self.pipeline.risk(text)
        if risk in ("HIGH", "CRITICAL"):
            if self.on_high_risk == "raise":
                raise EpistemicViolationError(self.pipeline.analyze(text))
            elif self.on_high_risk == "regulate":
                regulated = self.pipeline.regulate(text)
                if hasattr(response, "response"):
                    response.response = regulated
        return response
