"""Arithmetic solver for vCoT reasoning."""

from __future__ import annotations

import ast
import contextlib
import operator
import re

from ._clf import clf_predict as _clf_predict
from .chain import verify_chain
from .models import EpistemicChainReport

# ---------------------------------------------------------------------------
# Safe expression evaluator
# ---------------------------------------------------------------------------

_SAFE_OPS: dict[type, object] = {
    ast.Add: operator.add,
    ast.Sub: operator.sub,
    ast.Mult: operator.mul,
    ast.Div: operator.truediv,
    ast.FloorDiv: operator.floordiv,
    ast.Mod: operator.mod,
    ast.Pow: operator.pow,
    ast.USub: operator.neg,
    ast.UAdd: operator.pos,
}

_SAFE_NODE_TYPES = (
    ast.Expression,
    ast.BinOp,
    ast.UnaryOp,
    ast.Constant,
    ast.Add,
    ast.Sub,
    ast.Mult,
    ast.Div,
    ast.FloorDiv,
    ast.Mod,
    ast.Pow,
    ast.USub,
    ast.UAdd,
)


def _safe_eval(expr: str) -> float | None:
    """Evaluate a simple arithmetic expression without exec/eval."""
    try:
        tree = ast.parse(expr.strip(), mode="eval")
        for node in ast.walk(tree):
            if not isinstance(node, _SAFE_NODE_TYPES):
                return None

        def _eval(node: ast.expr) -> float | None:
            if isinstance(node, ast.Expression):
                return _eval(node.body)
            if isinstance(node, ast.Constant):
                return float(node.value)
            if isinstance(node, ast.BinOp):
                op_fn = _SAFE_OPS.get(type(node.op))
                if op_fn is None:
                    return None
                left_val, r = _eval(node.left), _eval(node.right)
                if left_val is None or r is None:
                    return None
                if isinstance(node.op, ast.Div) and r == 0:
                    return None
                return op_fn(left_val, r)  # type: ignore[operator]
            if isinstance(node, ast.UnaryOp):
                op_fn = _SAFE_OPS.get(type(node.op))
                val = _eval(node.operand)
                if op_fn is None or val is None:
                    return None
                return op_fn(val)  # type: ignore[operator]
            return None

        return _eval(tree)
    except Exception:
        return None


# ---------------------------------------------------------------------------
# Number extraction helpers
# ---------------------------------------------------------------------------


def _extract_numbers(text: str) -> list[float]:
    """Extract all numeric values from text, including word-form numbers.

    Digit-form numbers are extracted by regex.  Word-form numbers are
    converted token-by-token via ``word2number`` (handles "twenty-three",
    "a hundred", etc.).  Falls back silently if ``word2number`` is not
    installed.
    """
    nums: list[float] = []

    for m in re.finditer(r"-?\d+(?:[.,]\d+)?", text):
        with contextlib.suppress(ValueError):
            nums.append(float(m.group().replace(",", "")))

    try:
        from word2number import w2n

        for word in text.lower().split():
            word = word.strip(".,!?;:")
            # Skip digit-form tokens already captured by the regex above
            if word.lstrip("-").replace(".", "", 1).replace(",", "", 1).isdigit():
                continue
            with contextlib.suppress(ValueError):
                nums.append(float(w2n.word_to_num(word)))
    except ImportError:
        pass

    return nums


# ---------------------------------------------------------------------------
# Operation detection — classifier primary + structural dep-tree fallback
# ---------------------------------------------------------------------------


def _detect_operation(text: str) -> str | None:
    """Detect the primary arithmetic operation in a word problem.

    Strategy (in order):

    1. **Structural signals** scoped to the *question* sentence so that
       rate-giving premises (``per`` unit, ``each`` container) do not
       override the operation when the question asks for a total or product.
       Additional structural rules handle rate×count and rate-finding patterns.
    2. **Classifier** — TF-IDF + logistic regression on spaCy-lemmatized text
       (weights in ``data/op_classifier.npz``; pure-numpy inference).
    3. **Sentence-transformers** — optional zero-shot pass when ST is installed.

    No hardcoded verb lemma dict; all verb-level semantics are captured by the
    trained classifier.
    """
    import re as _re

    from ._z3utils import _get_nlp

    nlp = _get_nlp()
    doc = nlp(text.lower())
    all_lemmas: set[str] = {t.lemma_ for t in doc if not t.is_punct and not t.is_space}

    # Scope context-sensitive signals to the question sentence (last sentence).
    question_sent = _re.split(r"[.!]\s+", text.lower())[-1].strip()
    q_doc = nlp(question_sent)
    q_lemmas: set[str] = {t.lemma_ for t in q_doc if not t.is_punct and not t.is_space}

    # S1: "per" in question → asking for a rate → division
    if "per" in q_lemmas:
        return "/"

    # S2: "each" in question → asking for a per-unit amount → division
    if "each" in q_lemmas:
        return "/"

    # S3/S4: "per [unit]" in premise — check whether the question scales the
    # rate (multiplication) or asks for the denominator unit (division).
    per_match = _re.search(r"\bper\s+(\w+)", text.lower())
    if per_match:
        unit = per_match.group(1)
        # S3: "per X" + "for/in N X" in premise → rate × count → *
        if _re.search(r"\b(?:for|in)\s+\d+\s+" + unit + r"s?\b", text.lower()):
            return "*"
        # S4: "per X" in premise + "how many/much X" in question → total/rate → /
        unit_root = unit.rstrip("s") or unit
        if _re.search(r"\bhow\s+(?:many|much)\b.*\b" + unit_root + r"s?\b", question_sent):
            return "/"

    # S5: "total/sum/altogether" in question with no rate signal → additive sum
    if {"total", "sum", "altogether"} & q_lemmas and per_match is None:
        return "+"

    # 2. Classifier — verb-based semantics, pronouns excluded to prevent bias
    lemma_text = " ".join(
        t.lemma_ for t in doc if not t.is_punct and not t.is_space and t.pos_ != "PRON"
    )
    result = _clf_predict(lemma_text)
    if result is not None:
        return result

    # 3. Optional sentence-transformers zero-shot
    return _detect_operation_st(text)


_ST_EXEMPLARS: dict[str, str] = {
    "+": "How many items do they have altogether when combined?",
    "-": "How many are left after some were removed or spent?",
    "*": "How many total if each group contains the same number per unit?",
    "/": "What is the average amount each person receives when shared equally?",
}
_ST_CACHE: dict | None = None


def _detect_operation_st(text: str) -> str | None:
    """Zero-shot operation detection via sentence-transformers cosine similarity.

    Returns ``None`` (silently) when ``sentence_transformers`` or ``numpy``
    are not installed — the spaCy path is then the sole classifier.
    """
    global _ST_CACHE

    try:
        import numpy as np
        from sentence_transformers import SentenceTransformer

        if _ST_CACHE is None:
            model = SentenceTransformer("all-MiniLM-L6-v2")
            _ST_CACHE = {op: model.encode(sent) for op, sent in _ST_EXEMPLARS.items()}
            _ST_CACHE["_model"] = model

        model = _ST_CACHE["_model"]
        q = model.encode(text)
        scores = {
            op: float(np.dot(q, emb) / (np.linalg.norm(q) * np.linalg.norm(emb) + 1e-10))
            for op, emb in _ST_CACHE.items()
            if op != "_model"
        }
        return max(scores, key=scores.get)
    except Exception:
        return None


# ---------------------------------------------------------------------------
# Public API: solve_arithmetic
# ---------------------------------------------------------------------------


def solve_arithmetic(problem: str) -> EpistemicChainReport:
    """Solve a simple arithmetic word problem step-by-step with full verification.

    Generates a verified reasoning chain: extraction → operation → computation.
    Each step is verified by PureReason for internal consistency.

    Returns
    -------
    EpistemicChainReport where ``answer`` is the numeric result as a string,
    or a chain with failures if the problem cannot be parsed.
    """
    from ._z3utils import _get_nlp

    nums = _extract_numbers(problem)
    op = _detect_operation(problem)
    lower = problem.lower()

    steps: list[str] = []

    # Step 1: identify the given values
    if nums:
        vals_str = ", ".join(str(int(n) if n == int(n) else n) for n in nums[:4])
        steps.append(f"The problem gives us the following values: {vals_str}.")
    else:
        steps.append("The problem does not contain explicit numeric values.")

    # --- Multi-step 3-operand patterns (before standard 2-operand logic) ---
    answer_val: float | None = None

    if len(nums) >= 3:
        # Inverse proportion: "X workers ... N days ... how many days for M workers?"  → X*N/M
        # Detect via spaCy: animate-agent nouns + time nouns + "how many [time_noun]" question
        _agent_lemmas = {"worker", "person", "people", "machine", "employee", "man", "woman"}
        _time_lemmas = {"day", "hour", "week", "minute", "second"}
        doc_lc = _get_nlp()(lower)
        all_lc_lemmas = {t.lemma_ for t in doc_lc if not t.is_punct}
        if (
            _agent_lemmas & all_lc_lemmas
            and _time_lemmas & all_lc_lemmas
            and any(t.lemma_ == "many" for t in doc_lc)
            and _time_lemmas & {t.lemma_ for t in doc_lc if t.head.lemma_ == "many"}
        ):
            a, b, c = nums[0], nums[1], nums[2]
            if c != 0:
                answer_val = a * b / c
                steps.append(f"This is an inverse proportion: {a} × {b} / {c}.")
                fmt = str(int(answer_val)) if answer_val == int(answer_val) else f"{answer_val:.4g}"
                steps.append(f"Therefore, the answer is {fmt}.")
                return verify_chain(problem, steps)

        # Recipe/ratio scaling: "N X for M items, how many X for P items" → N/M*P
        # Detect "for" preposition followed by a NUM token via dep-tree
        doc_lc = _get_nlp()(lower)
        for_nums = [
            float(tok.text)
            for tok in doc_lc
            if (tok.pos_ == "NUM" and tok.head.lemma_ == "for")
            or (tok.pos_ == "NUM" and any(c.lemma_ == "for" for c in tok.head.children))
        ]
        if len(for_nums) >= 2:
            scale_base = float(for_nums[0])
            scale_target = float(for_nums[1])
            if scale_base != 0 and len(nums) >= 1:
                rate = nums[0] / scale_base
                answer_val = rate * scale_target
                steps.append(
                    f"This is a ratio/scaling problem: {nums[0]} / {scale_base} × {scale_target}."
                )
                fmt = str(int(answer_val)) if answer_val == int(answer_val) else f"{answer_val:.4g}"
                steps.append(f"Therefore, the answer is {fmt}.")
                return verify_chain(problem, steps)

    # Step 2: identify the operation (2-operand path)
    op_names = {"+": "addition", "-": "subtraction", "*": "multiplication", "/": "division"}
    if op and len(nums) >= 2:
        steps.append(
            f"The key operation is {op_names[op]} because the problem asks for "
            f"{'a total or sum' if op == '+' else 'a difference or remainder' if op == '-' else 'a product or rate' if op == '*' else 'a quotient, average, or rate'}."
        )
    else:
        steps.append("The operation cannot be determined without more information.")

    # Step 3: compute
    if op and len(nums) >= 2:
        a, b = nums[0], nums[1]
        if op == "/" and a < b and b != 0:
            a, b = b, a
        if op == "+":
            answer_val = a + b
        elif op == "-":
            answer_val = a - b
        elif op == "*":
            answer_val = a * b
        elif op == "/" and b != 0:
            answer_val = a / b

    if answer_val is not None:
        expr = f"{a} {op} {b}"  # type: ignore[possibly-undefined]
        computed = _safe_eval(expr)
        if computed is not None and abs(computed - answer_val) < 1e-9:
            fmt = str(int(answer_val)) if answer_val == int(answer_val) else f"{answer_val:.4g}"
            steps.append(f"Computing: {a} {op} {b} = {fmt}.")  # type: ignore[possibly-undefined]
            steps.append(f"Therefore, the answer is {fmt}.")
        else:
            steps.append("Arithmetic verification failed — computation is ambiguous.")
    else:
        steps.append("Cannot compute a definitive answer from the given values.")

    return verify_chain(problem, steps)
