# Contributing to PureReason

Thank you for helping make PureReason better. This document describes the
contribution process and the most impactful ways to contribute.

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Branching Strategy](#branching-strategy)
3. [Contributing World Prior Atlas Entries](#contributing-world-prior-atlas-entries)
4. [Code Contributions](#code-contributions)
5. [Reporting Benchmark Failures](#reporting-benchmark-failures)
6. [Development Setup](#development-setup)
7. [Testing](#testing)
8. [Code Style](#code-style)

---

## Quick Start

```bash
git clone https://github.com/sorunokoe/PureReason.git
cd PureReason
cargo build -p pure-reason-cli --release
cargo test
```

---

## Branching Strategy

We follow [GitHub Flow](https://docs.github.com/en/get-started/quickstart/github-flow):

| Branch | Purpose |
|---|---|
| `main` | Always releasable. CI must be green. Protected. |
| `feature/<name>` | New features — branch from `main`, PR back to `main` |
| `fix/<name>` | Bug fixes — branch from `main`, PR back to `main` |
| `prior/<id>` | World-prior atlas additions — branch from `main`, PR back to `main` |
| `release/vX.Y.Z` | Release preparation only (bump version, update changelog) |

**Rules:**
- Never commit directly to `main`
- Keep branches short-lived (days, not weeks)
- One logical change per PR
- Squash-merge preferred to keep `main` history clean

**Commit message format** (conventional commits):

```
<type>(<scope>): <short description>

<body>

Co-authored-by: ...
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `ci`, `chore`
Scopes: `core`, `cli`, `api`, `py`, `bench`, `reasoning`, `prior`

Examples:
```
feat(reasoning): add modus ponens detection to syllogism verifier
fix(bench): correct FELM oracle for arithmetic word problems
prior(science): add misconception about lightning rods
docs: update README benchmark table with LogicBench results
```

---

## Contributing World Prior Atlas Entries

The highest-impact contribution is expanding the misconception atlas. Every new
prior you add helps PureReason detect a category of hallucination it currently
misses — without any LLM cost.

### What makes a good prior?

A good prior:
- Is a **widely-believed factual misconception** that appears in real AI outputs
- Has **clear, testable signals** (specific keywords in both the myth and the correction)
- Has a **verifiable source** (Wikipedia, peer-reviewed study, government source)
- Is **unambiguous** — there is a clear consensus answer (not a disputed question)

### How to add a prior

1. Open `data/priors.yaml`
2. Find the right category section (or add one)
3. Add your entry in this format:

```yaml
- id: your_unique_snake_case_id
  category: science           # see category list in priors.yaml
  claim: "The wrong claim that AIs often produce"
  correct: "The accurate statement with clear explanation"
  topic_signals:              # keywords that must appear in the question/topic
    - "keyword1"
    - "keyword2"
  myth_signals:               # phrases that appear in the wrong answer (ANY matches)
    - "wrong phrase 1"
    - "wrong phrase 2"
  correction_signals:         # keywords present when the answer is CORRECT (ANY cancels the flag)
    - "correction keyword"
  confidence: 1.0             # use 0.9 if slightly uncertain
  source: "https://reliable-source.example.com"
```

4. Run the validator (if available):
```bash
python3 scripts/validate_priors.py
```

5. Test your prior:
```bash
./target/release/pure-reason calibrate "Question: <your topic> Answer: <the myth>"
./target/release/pure-reason calibrate "Question: <your topic> Answer: <the correction>"
# First should flag. Second should not.
```

6. Open a PR with title: `Prior: add <id>`

### Prior quality checklist

- [ ] ID is unique (check existing entries)
- [ ] `topic_signals` are specific enough to not fire on unrelated text
- [ ] `myth_signals` are phrases that actually appear in wrong AI answers
- [ ] `correction_signals` actually prevent false positives on correct answers
- [ ] Source is a reliable, accessible URL
- [ ] You verified the prior fires correctly with the CLI test above

---

## Code Contributions

### Good first issues

- Expanding the numeric plausibility atlas (constants, units)
- Adding test cases for edge cases in the Kantian detection layers
- Improving the dialogue epistemic state tracker (`dialogue.rs`)
- Language bindings / SDK improvements

### Architecture overview

```
crates/
├── pure-reason-core/   Core library — Kantian pipeline, ECS, all detectors
│   └── src/
│       ├── aesthetic/          Space/time input structuring
│       ├── analytic/           12 Kantian categories
│       ├── dialectic/          Illusion/antinomy/paralogism detection
│       ├── claims.rs           Claim IR + NanoType + ClaimTriple
│       ├── dialogue.rs         Dialogue epistemic state tracker
│       ├── calibration.rs      ECS scoring engine
│       ├── world_priors.rs     Misconception atlas (compiled from data/priors.yaml)
│       └── pipeline.rs         Full pipeline orchestrator
├── pure-reason-cli/    CLI interface (subcommands: calibrate, analyze, ...)
├── pure-reason-api/    REST API (axum)
├── pure-reason-mcp/    MCP server
└── pure-reason-py/     Python bindings (PyO3)

pureason/               Python package (pure-Python verifier layer)
├── reasoning/          vCoT engine — verify_chain, solve_arithmetic, verify_syllogism
├── guard.py            ReasoningGuard middleware (ECS + arithmetic repair)
└── integrations/       LangChain callback, @reasoning_guard decorator

tests/                  Unit tests (stdlib unittest, no binary required)
benchmarks/             Benchmark runners (heuristic-only, zero LLM)
```

### Before submitting a PR

1. Run Rust tests: `cargo test`
2. Run Python unit tests: `python3 -m unittest discover -s tests -p "test_*.py" -v`
3. Run ruff: `ruff check pureason/ && ruff format --check pureason/`
4. Run the local benchmark suite: `python3 benchmarks/run_benchmarks.py`
5. If changing detection logic: `python3 benchmarks/run_downloaded_benchmarks.py --n 25`

---

## Reporting Benchmark Failures

If you find a case where PureReason gets the wrong answer, please report it.
These failures drive the prior atlas expansion.

### How to report

Option 1 — Use the feedback CLI:
```bash
pure-reason feedback \
  --text "Your question and answer text" \
  --verdict wrong \
  --correct "The correct answer" \
  --category history
```

Option 2 — Open a GitHub Issue with label `benchmark-failure`:
```
Title: Failure: <brief description>

Input:
  Knowledge: <context if any>
  Question: <the question>
  Answer: <the answer PureReason got wrong>

PureReason verdict: hallucination / not hallucination (wrong)
Expected verdict: hallucination / not hallucination
Correct answer: <what is actually true>
Source: <URL>
```

---

## Development Setup

### Prerequisites

- Rust 1.70+ (`rustup update stable`)
- Python 3.9+ (for benchmark scripts and unit tests)

### Build the CLI binary

```bash
cargo build -p pure-reason-cli --release
```

### Install Python package (editable)

```bash
pip install -e ".[logic]"   # includes z3-solver for syllogism verification
pip install -e ".[dev]"     # adds ruff for linting
```

### Download benchmark datasets

```bash
python3 benchmarks/download_benchmarks.py --benchmarks halueval,ragtruth,truthfulqa,faithbench,felm
```

---

## Testing

### Run Python unit tests (no binary required)

```bash
python3 -m unittest discover -s tests -p "test_*.py" -v
```

### Run Rust unit tests

```bash
cargo test
```

### Run the local benchmark regression suite

```bash
python3 benchmarks/run_benchmarks.py
```

### Run the official benchmark evaluation (requires downloaded datasets)

```bash
# Quick (±8% CI, ~2 min)
python3 benchmarks/run_downloaded_benchmarks.py --n 25

# Standard (±4% CI, ~20 min)
python3 benchmarks/run_downloaded_benchmarks.py --n 100

# Release-quality (±3% CI, ~40 min)
python3 benchmarks/run_downloaded_benchmarks.py --n 200
```

### Run a specific benchmark

```bash
python3 benchmarks/run_downloaded_benchmarks.py --n 50 --benchmarks halueval_qa
```

---

## Code Style

- **Python**: use `ruff check` and `ruff format` (configured in `pyproject.toml`)
  - Max line length 100
  - Modern type hints (`str | None`, not `Optional[str]`)
  - No dead code, no commented-out blocks
- **Rust**: follow `rustfmt` defaults (`cargo fmt`)
- No `unwrap()` in library code — use `?` and the `PureReasonError` type
- Keep functions under 50 lines; extract helpers liberally
- Every new Rust module must have a `#[cfg(test)] mod tests { ... }` section
- Document public API with `///` (Rust) or docstrings (Python)

---

*PureReason is built on the principle that hallucination detection should be
deterministic, explainable, and free. Every contribution moves us closer to
the IFR: an epistemic calibration layer that is simply part of the AI ecosystem.*
