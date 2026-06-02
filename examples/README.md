# PureReason Examples

Practical, tested examples showing every major PureReason feature with
expected inputs, outputs, and integration patterns.

> **Prerequisite** — install the Python package first:
> ```bash
> pip install -e .                     # core (always required)
> pip install -e ".[nlp]"              # + spaCy & word2number
> python -m spacy download en_core_web_sm
> ```

---

## Use Cases at a Glance

| # | Example | File | What it covers |
|---|---------|------|----------------|
| 1 | **Guard Verification** | [`guard_verification.py`](guard_verification.py) | `ReasoningGuard` — ECS scoring, threshold tuning, arithmetic repair, degradation tracking |
| 2 | **Chain-of-Thought** | [`chain_of_thought.py`](chain_of_thought.py) | `verify_chain` — multi-step reasoning verification, contradiction detection |
| 3 | **Arithmetic Solver** | [`arithmetic_solver.py`](arithmetic_solver.py) | `solve_arithmetic` — word problem solving, number extraction, operation detection |
| 4 | **Syllogism Verification** | [`syllogism_verification.py`](syllogism_verification.py) | `verify_syllogism` — formal logic checking, fallacy detection (Z3 + heuristics) |
| 5 | **MCQ Picker** | [`mcq_picker.py`](mcq_picker.py) | `pick_best_answer` — multiple-choice selection, tie detection, strict mode |
| 6 | **Arithmetic Repair** | [`arithmetic_repair.py`](arithmetic_repair.py) | `_repair_arithmetic_in_step` — deterministic error correction, answer extraction, majority vote |
| 7 | **Simple Verification** | [`simple_verification.py`](simple_verification.py) | Quick-start 3-claim verification demo |
| 8 | **LangChain Integration** | [`langchain_integration.py`](langchain_integration.py) | `PureReasonVerifier` wrapper for LangChain pipelines |
| 9 | **API Server** | [`api_server.py`](api_server.py) | FastAPI microservice with `/verify` and `/verify/batch` |

---

## 1. Guard Verification — `ReasoningGuard`

The primary entry point. Verifies any text and returns an ECS score, provenance label, and optional arithmetic repair.

```bash
python examples/guard_verification.py
```

### Key Concepts

- **ECS (Epistemic Confidence Score)**: 0–100 score indicating how defensible a claim is.
- **Provenance**: One of `"verified"`, `"repaired"`, or `"flagged"`.
- **Threshold**: ECS below this → text is flagged (or repaired if arithmetic errors exist).

### Code

```python
from pureason.guard import ReasoningGuard

guard = ReasoningGuard(threshold=60, repair=True)
result = guard.verify("Water boils at 100°C at sea level.")

print(result.ecs)         # e.g. 75.0
print(result.provenance)  # "verified"
print(result.repaired)    # False
print(result.text)        # original text (unchanged)
```

### Decision Logic

```python
if result.ecs >= 70:
    action = "ACCEPT"
elif result.ecs >= 40:
    action = "REVIEW"
else:
    action = "REJECT"
```

### Arithmetic Repair

```python
result = guard.verify("3 + 4 = 8 so the total is wrong.")
# result.repaired == True
# result.text contains "= 7 [repaired]"
# result.original == "3 + 4 = 8 so the total is wrong."
```

### Degradation Tracking

```python
from pureason.guard import ReasoningGuard, _ReputationTracker

tracker = _ReputationTracker(window=5, baseline_window=20, drop=10.0)
guard = ReasoningGuard(threshold=60, source_label="my_model", tracker=tracker)

# After many verify() calls, if recent ECS drops >10 points below baseline,
# a ReasoningDegradationWarning is emitted.
```

---

## 2. Chain-of-Thought Verification — `verify_chain`

Verifies multi-step reasoning chains for internal consistency (each step alone)
and contextual consistency (each step against accumulated context).

```bash
python examples/chain_of_thought.py
```

### Code

```python
from pureason.reasoning import verify_chain

report = verify_chain(
    problem="A store has 50 apples. A customer buys 12. How many remain?",
    steps=[
        "The store starts with 50 apples.",
        "A customer buys 12 apples.",
        "Remaining = 50 - 12 = 38.",
        "Therefore, the answer is 38.",
    ],
)

print(report.is_valid)          # True — all steps pass
print(report.chain_confidence)  # harmonic mean of step ECS / 100
print(report.invalid_steps)     # [] — no failures
print(report.answer)            # last step text
print(report.summary)           # human-readable summary
```

### Detecting Arithmetic Errors in a Chain

```python
report = verify_chain("What is 15 + 27?", ["15 + 27 = 43."])
# Step 0 will have "ARITHMETIC_ERROR" in its flags
# because 15 + 27 = 42, not 43
```

### Edge Cases

```python
# Empty chain
report = verify_chain("Any?", [])
# report.is_valid == False, report.chain_confidence == 0.0

# Single step
report = verify_chain("What is 2 + 2?", ["2 + 2 = 4."])
# report.steps has 1 entry, report.answer == "2 + 2 = 4."
```

---

## 3. Arithmetic Solver — `solve_arithmetic`

Solves arithmetic word problems by extracting numbers, detecting the operation, computing the result, and verifying via a reasoning chain.

```bash
python examples/arithmetic_solver.py
```

### Building Blocks

```python
from pureason.reasoning.arithmetic import _safe_eval, _extract_numbers, _detect_operation

# Safe eval — no exec/eval, only arithmetic AST nodes
_safe_eval("(3 + 4) * 2")     # → 14.0
_safe_eval("import os")       # → None (rejected)
_safe_eval("5 / 0")           # → None (division by zero)

# Number extraction — digits, decimals, negatives, commas
_extract_numbers("There are 3 apples and 1,000 bananas.")  # → [3.0, 1000.0]

# Operation detection — NLP-based (spaCy + classifier)
_detect_operation("How many total after adding 5 more?")  # → "+"
_detect_operation("How many are left after removing 5?")   # → "-"
```

### Full Solver

```python
from pureason.reasoning import solve_arithmetic

report = solve_arithmetic(
    "Maria has 15 apples. She buys 8 more. How many apples in total?"
)
print(report.answer)    # "Therefore, the answer is 23."
print(report.is_valid)  # True
```

---

## 4. Syllogism Verification — `verify_syllogism`

Verifies logical arguments using a cascade of strategies:
1. TF-IDF + LogReg classifier (fast)
2. Z3 formal entailment (symbolic logic)
3. Informal fallacy heuristics
4. KAC semantic consistency (fallback)

```bash
python examples/syllogism_verification.py
```

### Code

```python
from pureason.reasoning import verify_syllogism

# Valid syllogism
report = verify_syllogism(
    premises=["All mammals are warm-blooded.", "Whales are mammals."],
    conclusion="Whales are warm-blooded.",
)
print(report.is_valid)  # True

# Invalid syllogism (undistributed middle)
report = verify_syllogism(
    premises=["All dogs are animals.", "All cats are animals."],
    conclusion="All dogs are cats.",
)
print(report.is_valid)  # False
```

### Report Structure

```python
report.is_valid          # bool — conclusion follows from premises
report.chain_confidence  # 0.88 (valid) or 0.25 (invalid)
report.summary           # human-readable explanation
report.steps             # StepVerification for each premise + conclusion
```

---

## 5. MCQ Picker — `pick_best_answer`

Selects the best answer from multiple choices by verifying each against the question context.

```bash
python examples/mcq_picker.py
```

### Code

```python
from pureason.reasoning import pick_best_answer

best_idx, report = pick_best_answer(
    question="What is the capital of France?",
    choices=["Berlin", "Paris", "Madrid", "Rome"],
)
print(f"Best: {best_idx}")  # index of highest-ECS choice

# With context
best_idx, report = pick_best_answer(
    question="Which animal is fastest?",
    choices=["Cheetah (70 mph)", "Lion (50 mph)", "Elephant (25 mph)"],
    context="African wildlife guide.",
)

# Strict mode — raises AmbiguousAnswerError on ties
from pureason.reasoning.mcq import AmbiguousAnswerError
try:
    pick_best_answer("Pick one.", ["Red", "Blue"], strict=True)
except AmbiguousAnswerError as e:
    print(f"Tied: {e.tied_indices}")
```

---

## 6. Arithmetic Repair — `_repair_arithmetic_in_step`

Deterministic repair of arithmetic errors in text. Finds `A op B = C` patterns and corrects wrong results.

```bash
python examples/arithmetic_repair.py
```

### Code

```python
from pureason.reasoning.repair import _repair_arithmetic_in_step

# Correct — no change
_repair_arithmetic_in_step("3 + 4 = 7 apples.")
# → "3 + 4 = 7 apples."

# Wrong — repaired
_repair_arithmetic_in_step("3 + 4 = 8 apples.")
# → "3 + 4 = 7 [repaired] apples."

# Extraction utilities
from pureason.reasoning.repair import _extract_numeric_answer, _extract_letter_answer
_extract_numeric_answer("The answer is 42.")  # → 42.0
_extract_letter_answer("The answer is **B**.")  # → "B"

# Majority vote for aggregating multiple answers
from pureason.reasoning.repair import _majority_vote, _majority_vote_letters
_majority_vote([42.0, 42.0, 41.0])  # → 42.0
_majority_vote_letters(["A", "B", "A"])  # → "A"
```

---

## 7–9. Quick Start, LangChain, API Server

These examples are documented inline:

- **[`simple_verification.py`](simple_verification.py)** — 3-claim quickstart
- **[`langchain_integration.py`](langchain_integration.py)** — `PureReasonVerifier` wrapper
- **[`api_server.py`](api_server.py)** — FastAPI server (`pip install fastapi uvicorn`)

---

## 🐳 Docker Deployment

```bash
docker build -f examples/Dockerfile.api -t pureason-api .
docker run -p 8000:8000 pureason-api
curl http://localhost:8000/health
```

## API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Health check |
| `/metrics` | GET | Performance metrics |
| `/verify` | POST | Verify single claim |
| `/verify/batch` | POST | Verify up to 100 claims |
| `/docs` | GET | Interactive API docs |

---

## Configuration

### ECS Thresholds

| Risk Level | Domain | Min ECS |
|------------|--------|---------|
| **Critical** | Medical, Legal, Financial | 85+ |
| **High** | Business, Education | 75+ |
| **Medium** | General knowledge | 65+ |
| **Low** | Creative, Opinion | 50+ |

### Integration Patterns

**Guard Rails** — reject low-confidence output:
```python
guard = ReasoningGuard(threshold=70)
result = guard.verify(llm_output)
if result.provenance == "flagged":
    raise ValueError("Output failed verification")
```

**Confidence Scoring** — attach scores to outputs:
```python
result = guard.verify(llm_output)
return {"text": llm_output, "confidence": result.ecs / 100}
```

**Auto-Correction** — repair arithmetic mistakes:
```python
result = guard.verify(llm_output)
if result.repaired:
    return result.text  # corrected version
return llm_output
```

---

## Tests

All examples have corresponding tests in [`tests/test_examples.py`](../tests/test_examples.py).

```bash
python -m pytest tests/test_examples.py -v
```

## Troubleshooting

**"ModuleNotFoundError: No module named 'pureason'"**
```bash
pip install -e .
```

**"PureReason reasoning requires spaCy"**
```bash
pip install -e ".[nlp]"
python -m spacy download en_core_web_sm
```

**"API server won't start"**
```bash
pip install fastapi uvicorn pydantic
```
