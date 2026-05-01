"""Arithmetic verification oracles for FELM, RAGTruth, and FaithBench (S47/S48/S26)."""

import ast
import contextlib
import operator as _op
import re

# ─── S47 Arithmetic Chain Verifier ───────────────────────────────────────────

_ARITH_RE = re.compile(
    r"(-?\d+(?:\.\d+)?)\s*([+\-×x*÷/])\s*(-?\d+(?:\.\d+)?)\s*=\s*(-?\d+(?:\.\d+)?)",
    re.IGNORECASE,
)
_PCT_RE = re.compile(
    r"(\d+(?:\.\d+)?)\s*(?:%|percent)\s+of\s+(\d+(?:\.\d+)?)\s+"
    r"(?:is|=|equals|was|are)\s+(\d+(?:\.\d+)?)",
    re.IGNORECASE,
)


def arithmetic_error_oracle(text: str) -> bool:
    """Detect explicit arithmetic errors in any text block (S47).

    Checks two patterns:
    1. 'A op B = C' for op in +, -, *, /  (e.g. '5 + 3 = 9' → error)
    2. 'X% of Y is Z'                      (e.g. '50% of 200 is 120' → error)

    Uses relative tolerance 0.5% to handle floating-point representations.
    Returns True (ISSUE) on first detected error.
    """
    for m in _ARITH_RE.finditer(text):
        a, op, b, claimed = (
            float(m.group(1)),
            m.group(2).lower(),
            float(m.group(3)),
            float(m.group(4)),
        )
        if op == "+":
            expected = a + b
        elif op == "-":
            expected = a - b
        elif op in ("*", "x", "×"):
            expected = a * b
        elif op in ("/", "÷"):
            if b == 0:
                continue
            expected = a / b
        else:
            continue
        tol = max(1e-6, abs(expected) * 0.005)
        if abs(expected - claimed) > tol:
            return True

    for m in _PCT_RE.finditer(text):
        pct, base, claimed = float(m.group(1)), float(m.group(2)), float(m.group(3))
        expected = pct * base / 100
        tol = max(1e-6, abs(expected) * 0.005)
        if abs(expected - claimed) > tol:
            return True

    return False


# ─── S48 Per-sentence Self-consistency Checker ───────────────────────────────

_SENT_SPLIT = re.compile(r"(?<=[.!?])\s+(?=[A-Z])")


def _extract_quant_map(text: str) -> dict[str, float]:
    """Extract {entity/quantity_label → value} from a sentence."""
    result: dict[str, float] = {}
    for m in re.finditer(
        r"([A-Za-z][\w\s]{1,20}?)\s+(?:has|have|is|are|costs?|equals?|=|totals?)\s+(-?\d+(?:\.\d+)?)",
        text,
        re.IGNORECASE,
    ):
        label = m.group(1).strip().lower()
        val = float(m.group(2))
        result[label] = val
    return result


def reasoning_chain_consistency_oracle(text: str) -> bool:
    """Detect numerical self-contradictions in multi-sentence answers (S48).

    Splits answer into sentences, builds a quantity map for each, then checks
    if any later sentence assigns a DIFFERENT value to the same label.

    Example (caught): 'John has 5 apples. ... John has 8 apples.'
    """
    ans_m = re.search(r"\n(?:Answer|Response|Conclusion):\s*(.+?)(?:\n\n|$)", text, re.DOTALL)
    if not ans_m:
        return False

    answer = ans_m.group(1)
    sentences = _SENT_SPLIT.split(answer)
    if len(sentences) < 2:
        return False

    accumulated: dict[str, float] = {}
    for sent in sentences:
        sent_map = _extract_quant_map(sent)
        for label, val in sent_map.items():
            if label in accumulated:
                prev = accumulated[label]
                tol = max(1e-6, abs(prev) * 0.01)
                if abs(prev - val) > tol:
                    return True
            accumulated[label] = val

    return False


# ─── TRIZ S3 NPD Extension: AST arithmetic evaluator for FELM ────────────────

_SAFE_AST_NODES = (
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
_OP_MAP = {
    ast.Add: _op.add,
    ast.Sub: _op.sub,
    ast.Mult: _op.mul,
    ast.Div: _op.truediv,
    ast.FloorDiv: _op.floordiv,
    ast.Mod: _op.mod,
    ast.Pow: _op.pow,
}


def _safe_eval(expr: str) -> float | None:
    """Safely evaluate a pure numeric arithmetic expression via AST walking."""
    expr = re.sub(r",(?=\d{3})", "", expr).replace("×", "*").replace("÷", "/").replace("^", "**")
    try:
        tree = ast.parse(expr.strip(), mode="eval")
        if not all(isinstance(n, _SAFE_AST_NODES) for n in ast.walk(tree)):
            return None

        def _eval(node):
            if isinstance(node, ast.Expression):
                return _eval(node.body)
            if isinstance(node, ast.Constant) and isinstance(node.value, (int, float)):
                return float(node.value)
            if isinstance(node, ast.BinOp) and type(node.op) in _OP_MAP:
                return _OP_MAP[type(node.op)](_eval(node.left), _eval(node.right))
            if isinstance(node, ast.UnaryOp) and isinstance(node.op, (ast.USub, ast.UAdd)):
                v = _eval(node.operand)
                return -v if isinstance(node.op, ast.USub) else v
            raise ValueError(f"unsupported node: {type(node)}")

        return _eval(tree)
    except Exception:
        return None


def _arithmetic_error_in_felm(text: str) -> bool:
    """Detect arithmetic errors in FELM text by extracting and verifying expressions.

    Extracts the FULL arithmetic expression from the prompt, evaluates it with
    the safe AST evaluator (respecting operator precedence), then checks whether
    the correct result appears in the response.
    TRIZ S3 NPD Extension: arithmetic verification as numeric plausibility check.
    """
    if "\nAnswer:" in text:
        prompt_part, response_part = text.split("\nAnswer:", 1)
    elif "\n" in text:
        parts = text.split("\n", 1)
        prompt_part, response_part = parts[0], parts[1]
    else:
        return False

    clean_prompt = (
        re.sub(r",(?=\d{3})", "", prompt_part)
        .replace("×", "*")
        .replace("÷", "/")
        .replace("^", "**")
    )
    clean_response = re.sub(r",(?=\d{3})", "", response_part)

    candidates = re.findall(r"[\d\s+\-*/().]+(?:\*\*[\d\s()+\-*/().]+)?", clean_prompt)
    if not candidates:
        return False

    correct = None
    for candidate in sorted(candidates, key=len, reverse=True):
        stripped = candidate.strip()
        if not re.search(r"[+\-*/]", stripped):
            continue
        val = _safe_eval(stripped)
        if val is not None and abs(val) <= 1e15:
            correct = val
            break

    if correct is None:
        return False

    resp_nums: set[float] = set()
    for raw in re.findall(r"-?\d+(?:\.\d+)?", clean_response):
        with contextlib.suppress(ValueError):
            resp_nums.add(float(raw))

    if not resp_nums:
        return False

    if correct == int(correct):
        return int(correct) not in {int(n) for n in resp_nums if n == int(n)}
    tol = abs(correct) * 1e-4 + 1e-9
    return not any(abs(n - correct) <= tol for n in resp_nums)


# ─── FELM Track 1 — Word Problem Extractor (S26) ─────────────────────────────

_WORD_PROBLEM_PATTERNS = [
    (
        re.compile(
            r"(?:has|have|start(?:s|ed)? with|begin(?:s|ning)? with|owns?|contains?|holds?)"
            r"\s+(\d[\d,]*(?:\.\d+)?)\s+\w+[^.]*?"
            r"(?:gives?|give away|loses?|sells?|spends?|removes?|takes? away|donates?|uses?)\s+(\d[\d,]*(?:\.\d+)?)",
            re.IGNORECASE,
        ),
        lambda g: float(g[0].replace(",", "")) - float(g[1].replace(",", "")),
        "subtraction_word",
    ),
    (
        re.compile(
            r"(\d[\d,]*(?:\.\d+)?)\s+\w+[^.]*?"
            r"(?:and|then|also|plus|adds?|buys?|receives?|gains?|gets?)\s+(\d[\d,]*(?:\.\d+)?)",
            re.IGNORECASE,
        ),
        lambda g: float(g[0].replace(",", "")) + float(g[1].replace(",", "")),
        "addition_word",
    ),
    (
        re.compile(
            r"(\d[\d,]*(?:\.\d+)?)\s+(?:times|multiplied by|×)\s+(\d[\d,]*(?:\.\d+)?)",
            re.IGNORECASE,
        ),
        lambda g: float(g[0].replace(",", "")) * float(g[1].replace(",", "")),
        "multiplication_word",
    ),
    (
        re.compile(
            r"(\d[\d,]*(?:\.\d+)?)\s+(?:divided by|÷)\s+(\d[\d,]*(?:\.\d+)?)",
            re.IGNORECASE,
        ),
        lambda g: (
            float(g[0].replace(",", "")) / float(g[1].replace(",", ""))
            if float(g[1].replace(",", "")) != 0
            else None
        ),
        "division_word",
    ),
    (
        re.compile(
            r"(\d[\d,]*(?:\.\d+)?)\s+\w+\s+and\s+(\d[\d,]*(?:\.\d+)?)\s+\w+"
            r"[^.]*?(?:total|altogether|in all|combined|sum)",
            re.IGNORECASE,
        ),
        lambda g: float(g[0].replace(",", "")) + float(g[1].replace(",", "")),
        "total_word",
    ),
]


def _word_problem_error_in_felm(text: str) -> bool:
    """Detect arithmetic errors in FELM word problems (S26).

    Applies word-problem pattern matching to extract quantities and operations,
    then verifies the claimed answer against the computed result.
    Returns False if no word problem is detected (fail-safe: no false positives).
    """
    if "\nAnswer:" in text:
        prompt_part, response_part = text.split("\nAnswer:", 1)
    elif "\n" in text:
        parts = text.split("\n", 1)
        prompt_part, response_part = parts[0], parts[1]
    else:
        prompt_part, response_part = text, ""

    clean_response = re.sub(r",(?=\d{3})", "", response_part)

    for pattern, compute_fn, _name in _WORD_PROBLEM_PATTERNS:
        match = pattern.search(prompt_part)
        if match:
            try:
                correct = compute_fn(match.groups())
                if correct is None or not (0 < abs(correct) < 1e12):
                    continue
            except (ZeroDivisionError, ValueError, IndexError):
                continue

            resp_nums: set[float] = set()
            for raw in re.findall(r"-?\d+(?:\.\d+)?", clean_response):
                with contextlib.suppress(ValueError):
                    resp_nums.add(float(raw))

            if not resp_nums:
                continue

            if correct == int(correct) if correct == correct else False:
                if int(correct) not in {int(n) for n in resp_nums if n == int(n)}:
                    return True
            else:
                tol = abs(correct) * 1e-3 + 1e-9
                if not any(abs(n - correct) <= tol for n in resp_nums):
                    return True

    return False
