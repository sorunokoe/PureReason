"""S106-C: Universal @reasoning_guard decorator.

Wraps any function that returns an LLM string response with PureReason
verification. No changes to existing code beyond the decorator line.

Usage
-----
    from pureason.integrations.decorator import reasoning_guard

    @reasoning_guard(threshold=65, repair=True)
    def ask_llm(prompt: str) -> str:
        return openai.chat.completions.create(
            model="gpt-4o",
            messages=[{"role": "user", "content": prompt}],
        ).choices[0].message.content

    # Now every call to ask_llm() has its output verified:
    result = ask_llm("What is 2 + 2?")
    # result may be "[guard:verified] 4" or "[guard:repaired] 4" etc.

The decorator works with any function that returns a string, regardless
of which LLM provider is used. It is provider-agnostic by design.
"""

from __future__ import annotations

import functools
from typing import Callable


def reasoning_guard(
    threshold: int = 60,
    repair: bool = True,
    verbose: bool = False,
) -> Callable:
    """Decorator factory: verify LLM string output with PureReason.

    Parameters
    ----------
    threshold : int
        ECS score below which PureReason intervenes (0–100). Default 60.
    repair : bool
        Attempt arithmetic repair on low-ECS outputs. Default True.
    verbose : bool
        Print ECS score and provenance to stdout for each call. Default False.

    Returns
    -------
    Decorator that wraps any function returning a str.
    """

    def decorator(fn: Callable) -> Callable:
        @functools.wraps(fn)
        def wrapper(*args, **kwargs) -> str:
            result = fn(*args, **kwargs)
            if not isinstance(result, str):
                return result  # non-string output: pass through unchanged

            try:
                from pureason.reasoning import _ecs_score, _repair_arithmetic_in_step

                ecs = float(_ecs_score(result))
                if verbose:
                    print(
                        f"  [PureReason @{fn.__name__}] ECS={ecs:.0f}",
                        flush=True,
                    )

                if ecs >= threshold:
                    return result  # fast path

                if repair:
                    repaired = _repair_arithmetic_in_step(result)
                    if repaired != result:
                        return f"[guard:repaired] {repaired}"

                return f"[guard:flagged ecs={ecs:.0f}] {result}"
            except Exception:
                return result  # never block on guard failure

        return wrapper

    return decorator
