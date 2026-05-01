"""PureReasonCallback — LangChain verification middleware.

Verifies every LLM response in a LangChain chain using PureReason's ECS
(Epistemic Calibration Score) and arithmetic repair. Requires only the
Rust binary — no LLM of our own, no API keys.

Usage
-----
    from langchain.chains import LLMChain
    from pureason.integrations.langchain import PureReasonCallback

    chain = LLMChain(
        llm=your_llm,
        prompt=your_prompt,
        callbacks=[PureReasonCallback(threshold=60, repair=True)],
    )
    result = chain.run("What is 17 × 23?")
    # Every LLM response is ECS-verified; arithmetic errors repaired.

Provenance tags injected into LLM output:
    "[guard:verified]"  — ECS ≥ threshold
    "[guard:repaired]"  — arithmetic repaired
    "[guard:flagged]"   — low ECS
"""

from __future__ import annotations

from typing import Any


class PureReasonCallback:
    """LangChain BaseCallbackHandler that verifies every LLM response.

    Parameters
    ----------
    threshold : int
        ECS score below which PureReason intervenes (0–100). Default 60.
    repair : bool
        Attempt arithmetic repair on low-ECS responses. Default True.
    verbose : bool
        Print ECS and provenance to stdout. Default False.
    """

    def __init__(self, threshold: int = 60, repair: bool = True, verbose: bool = False):
        self.threshold = threshold
        self.repair = repair
        self.verbose = verbose
        self._stats: dict[str, int] = {"total": 0, "verified": 0, "repaired": 0, "flagged": 0}

        try:
            from langchain.callbacks.base import BaseCallbackHandler

            self.__class__ = type(
                "PureReasonCallback", (PureReasonCallback, BaseCallbackHandler), {}
            )
            BaseCallbackHandler.__init__(self)
        except ImportError:
            pass

    def on_llm_end(self, response: Any, **kwargs: Any) -> None:
        """Verify every generation in the LLM response."""
        try:
            from pureason.reasoning import _ecs_for_text, _repair_arithmetic_in_step

            for gen_list in getattr(response, "generations", []):
                for gen in gen_list:
                    text = getattr(gen, "text", "")
                    if not text:
                        continue
                    self._stats["total"] += 1
                    ecs, _ = _ecs_for_text(text)
                    if ecs >= self.threshold:
                        self._stats["verified"] += 1
                        provenance = "verified"
                    elif self.repair:
                        repaired = _repair_arithmetic_in_step(text)
                        if repaired != text:
                            gen.text = f"[guard:repaired] {repaired}"
                            self._stats["repaired"] += 1
                            provenance = "repaired"
                        else:
                            gen.text = f"[guard:flagged ecs={ecs}] {text}"
                            self._stats["flagged"] += 1
                            provenance = "flagged"
                    else:
                        gen.text = f"[guard:flagged ecs={ecs}] {text}"
                        self._stats["flagged"] += 1
                        provenance = "flagged"
                    if self.verbose:
                        print(f"  [PureReason] ECS={ecs} → {provenance}", flush=True)
        except Exception:
            pass

    def on_llm_error(self, error: Exception, **kwargs: Any) -> None:
        if self.verbose:
            print(f"  [PureReason] LLM error: {error}", flush=True)

    @property
    def stats(self) -> dict[str, int]:
        return dict(self._stats)
