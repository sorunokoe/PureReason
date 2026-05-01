"""
AutoGen integration for PureReason (S-IV-2).

Provides a reply function that validates agent messages before delivery.

Usage:
    from pure_reason.autogen import kantian_reply_func
    import autogen

    assistant = autogen.AssistantAgent(
        name="assistant",
        reply_functions=[kantian_reply_func(domain="medical")],
    )
"""

from __future__ import annotations

from typing import Any, Callable

from .exceptions import EpistemicViolationError
from .pipeline import Pipeline


def kantian_reply_func(
    domain: str = "general",
    on_high_risk: str = "regulate",
    api_url: str | None = None,
) -> Callable:
    """
    Returns an AutoGen reply function that validates agent messages.

    The returned function can be registered via agent.register_reply() or
    passed in the reply_functions list.

    Args:
        domain: Domain profile.
        on_high_risk: 'warn' | 'regulate' | 'raise'.
        api_url: Override REST API URL.
    """
    pipeline = Pipeline(domain=domain, api_url=api_url)

    def validate_reply(
        recipient: Any,
        messages: Any,
        sender: Any,
        config: Any,
    ) -> tuple[bool, str | None]:
        """AutoGen reply function signature."""
        if not messages:
            return False, None

        last_message = messages[-1] if isinstance(messages, list) else messages
        text = (
            last_message.get("content", "") if isinstance(last_message, dict) else str(last_message)
        )

        if not text or not text.strip():
            return False, None

        result = pipeline.validate(text)
        risk = result.get("risk_level", "SAFE")

        if risk in ("HIGH", "CRITICAL"):
            if on_high_risk == "raise":
                raise EpistemicViolationError(pipeline.analyze(text))
            elif on_high_risk == "regulate":
                regulated = pipeline.regulate(text)
                return True, regulated
            else:
                import warnings

                warnings.warn(f"PureReason: {risk} risk in AutoGen message.", stacklevel=2)

        return False, None  # Pass through unchanged

    return validate_reply
