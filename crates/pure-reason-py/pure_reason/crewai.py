"""
CrewAI integration for PureReason (S-IV-2).

Provides validated task and agent wrappers.

Usage:
    from pure_reason.crewai import kantian_task, KantianAgent
    from crewai import Crew

    task = kantian_task(
        description="Analyze the patient's symptoms",
        domain="medical",
        on_high_risk="regulate",
    )
"""

from __future__ import annotations

from typing import Any, Callable

from .exceptions import EpistemicViolationError
from .pipeline import Pipeline


def kantian_task(
    description: str,
    domain: str = "general",
    on_high_risk: str = "warn",
    original_callback: Callable | None = None,
    **kwargs: Any,
) -> Any:
    """
    Factory for a CrewAI Task with epistemic output validation.

    Args:
        description: The task description.
        domain: Domain profile for validation.
        on_high_risk: 'warn' | 'regulate' | 'raise'.
        original_callback: Optional existing callback to chain.
        **kwargs: Additional arguments passed to crewai.Task.

    Returns:
        crewai.Task with Kantian validation callback attached.
    """
    pipeline = Pipeline(domain=domain)

    def validated_callback(output: Any) -> Any:
        text = str(output)
        result = pipeline.validate(text)
        risk = result.get("risk_level", "SAFE")

        if risk in ("HIGH", "CRITICAL"):
            if on_high_risk == "raise":
                raise EpistemicViolationError(pipeline.analyze(text))
            elif on_high_risk == "regulate":
                regulated = pipeline.regulate(text)
                # Try to mutate the output object
                if hasattr(output, "raw_output"):
                    output.raw_output = regulated
                elif hasattr(output, "output"):
                    output.output = regulated
            else:
                import warnings

                warnings.warn(
                    f"PureReason: {risk} epistemic risk in CrewAI task output. "
                    f"Summary: {result.get('summary', '')}",
                    stacklevel=2,
                )

        if original_callback:
            original_callback(output)
        return output

    try:
        from crewai import Task

        return Task(description=description, callback=validated_callback, **kwargs)
    except ImportError:
        # crewai not installed — return a plain dict describing the task
        return {"description": description, "callback": validated_callback, **kwargs}
