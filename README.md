<div align="center">

<pre>
╔═════════════════════════════╗
║                             ║
║ ◈  P U R E   R E A S O N  ◈ ║
║                             ║
╚═════════════════════════════╝
</pre>

[![Version](https://img.shields.io/badge/version-0.3.1-blue.svg)](CHANGELOG.md)
[![CI](https://github.com/sorunokoe/PureReason/actions/workflows/lint.yml/badge.svg)](https://github.com/sorunokoe/PureReason/actions/workflows/lint.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-693%20passing-brightgreen.svg)](#testing)
[![MCP](https://img.shields.io/badge/integration-MCP%2FCLI-blue.svg)](#default-operating-mode)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](./.github/CONTRIBUTING.md)

**✨ NEW: [v0.3.1 Release](#whats-new-in-v030) – Neural Models Implementation**

</div>

---

**PureReason** is a deterministic reasoning assurance engine for agentic workflows.
It is strongest when used to **verify, calibrate, and govern** model output rather
than replace frontier models outright. The core system scores text with a
**0-100 Epistemic Confidence Score (ECS)**, checks reasoning chains, flags
contradictions and overconfidence, and rewrites risky output into a more
defensible form.

The primary deployment flow is:

- the user talks to **Claude Code, Copilot, Codex, or another frontier agent**
- that agent calls **PureReason** through MCP, the local CLI, or a loopback-local service
- PureReason returns verification findings, regulated text, task state, and traceable review decisions

PureReason is **not** trying to replace the agent's chat loop or own model access by default.

## Why this exists in the agentic era

Frontier models are powerful, but real multi-step workflows still struggle with:

- non-determinism that hurts debugging and trust
- weak auditability for tool use and long-horizon decisions
- cost and latency blowups in chained agent runs
- brittle tool contracts, loop/stuck states, and prompt-injection risk
- poor review and escalation paths for high-risk actions

PureReason's direction is to become the **reasoning assurance layer** around
those systems:

- integrate with frontier agents through MCP, CLI, and local review contracts
- verify plans, tool calls, and outputs before action
- attach evidence, provenance, and reviewable traces
- support human approval flows where risk is high
- keep local/default workflows usable without provider credentials
- start with **software plan/spec review** and **software change review** for AI
  platform and engineering teams

## Best current fit

- existing coding agents that need a local verifier/reviewer they can call directly
- verifying model output instead of generating it
- local or offline reasoning checks
- auditability and policy enforcement around sensitive workflows
- arithmetic, logic, contradiction, and overconfidence detection

## Default operating mode

PureReason now has a single primary operating mode:

- **local assurance** — Pure Rust / Z3 / regex / verifier-runtime stack, exposed through MCP and CLI

That default path is designed to work without provider credentials because the
frontier model already lives in the agent that is calling PureReason.

Local review state is persisted by default under `~/.pure-reason/agent-state/`
as `tasks.sqlite3`, `traces.sqlite3`, and `evidence.sqlite3`. Override the
directory with `PURE_REASON_STATE_DIR`, or with
`pure-reason review --state-dir ...` for one-off CLI runs. If no home directory
can be resolved, `PURE_REASON_STATE_DIR` must be set explicitly.

---

## Quick Start (MCP Integration)

**For frontier agents (Claude Code, GitHub Copilot, Cursor)**:

```bash
# 1. Build the MCP server
cargo build --release -p pure-reason-mcp

# 2. Add to your agent's MCP config (e.g., Claude Desktop)
# See docs/MCP-INTEGRATION.md for detailed setup

# 3. Agent can now call PureReason tools:
#    - verify_text / verify_structured_decision
#    - review_text / review_structured_decision  
#    - analyze / certify / regulate / validate
```

**Standalone CLI**:

```bash
cargo install --path crates/pure-reason-cli --locked
pure-reason review "The patient must have cancer."
```

**Full integration guide**: [`docs/MCP-INTEGRATION.md`](./docs/MCP-INTEGRATION.md)

---

## What you get today

### Core Features
- **ECS score (0-100)** with confidence banding and domain-aware calibration
- **Reasoning-chain verification** for arithmetic and logical steps
- **Contradiction and hallucination-like risk detection**
- **Domain-aware regulation/rewrite** for overconfident or risky language
- **Agent-facing review surfaces**: MCP tools and `pure-reason review` for local agent calls
- **Durable local review state**: persisted task, trace, and review-evidence stores for agent workflows
- **Multiple product surfaces**: CLI, Rust library, Python wrapper, REST API, **MCP**
- **Deterministic local mode** designed to complement existing frontier agents

### What's New in v0.3.0

PureReason now includes **systematic performance improvements** across the verification pipeline:

- **Pre-Verification Gate V2** — Fast pre-checks (<5ms) short-circuit 60-80% of simple claims
  - Arithmetic error detection (<1ms)
  - Blacklist pattern matching
  - Input complexity scoring
  - **Impact**: -40% average latency

- **Session Meta-Learner V2** — Adaptive learning adjusts detector weights based on accuracy
  - Session-scoped (no cross-session contamination)
  - Per-detector accuracy tracking
  - 100-call warmup period
  - **Impact**: +5-10pp F1 after warmup

- **Domain Calibration** — Per-domain ensemble weights and ECS calibration
  - Regex-based domain detection (medical, legal, financial, general)
  - YAML configuration per domain
  - Platt scaling calibration curves
  - **Impact**: ±5pp ECS accuracy (vs ±15pp before)

- **Wikipedia Corpus** — 6M Wikipedia article knowledge base with BM25 search
  - Lazy loading SQLite FTS5 index
  - Entity detection for novelty checking
  - LRU cache for performance
  - **Impact**: +18pp TruthfulQA recall (when corpus available)

- **Semantic Fallback** — Embedding-based hallucination detection using all-MiniLM-L6-v2
  - Cosine similarity threshold detection (<0.86 = hallucination)
  - Catches semantic variations pattern matching misses
  - Python subprocess interface with graceful fallback
  - **Status**: Fully implemented, optimizations pending
  - **Impact**: +8-12pp recall on narrative hallucinations

**Cumulative gains**: +25-30pp F1, -40% latency, 3× better calibration accuracy

See [`docs/TRIZ-IMPLEMENTATION.md`](./docs/TRIZ-IMPLEMENTATION.md) for full guide.

### What PureReason is NOT

- **Not a general-purpose frontier reasoning model**
- **Not a problem solver** — we verify and score solutions, we do not generate them
- **Not a content generator** — we assess confidence in existing text
- **Not an LLM** — PureReason is the verifier/reviewer around an existing agent
- **Not a replacement for domain expertise** — we flag risk, human judgment required
- **Not yet a full agent runtime** — the current repo is stronger at verification
  than orchestration
- **Not for long-context** — optimized for claim-level reasoning (<10K tokens
  vs LLM 100K+)

## Evidence and roadmap

When you want measured capabilities and caveats, start with:

- [`docs/CAPABILITIES.md`](./docs/CAPABILITIES.md)
- [`docs/BENCHMARK.md`](./docs/BENCHMARK.md)
- [`docs/METHODOLOGY.md`](./docs/METHODOLOGY.md)
- [`docs/REPRODUCIBILITY.md`](./docs/REPRODUCIBILITY.md)

When you want the product direction, read:

- [`docs/ADR-002.md`](./docs/ADR-002.md) — agentic reasoning assurance roadmap
- [`docs/README.md`](./docs/README.md) — documentation index and evidence-first reading order

### Example signal

```text
Input:  "The patient must have cancer."
ECS:    34/100 (LOW)
Flag:   Certainty overreach
Rewrite:"The patient has findings consistent with possible malignancy."
```

### Chain-of-thought check (math reasoning)

```text
Step 1: A train travels 120 miles in 2 hours.
Step 2: Speed = 120 / 2 = 90 mph
Step 3: Time for 300 miles = 300 / 90 ≈ 3.3 hours
```

```text
Result: INVALID
First failing step: 2
Reason: arithmetic_error (120 / 2 should be 60, not 90)
```

PureReason verifies each step deterministically and pinpoints the exact failure, so you get traceable reasoning checks instead of a black-box verdict.

---

## Python Reasoning Layer

The Python layer provides formal logic verification, arithmetic reasoning, and
MCQ evaluation on top of the Rust core.

### Installation

```bash
# Base install (pure Rust scoring only)
pip install -e .

# With NLP reasoning (spaCy dependency parsing, word-to-number)
pip install -e ".[nlp]"
python -m spacy download en_core_web_sm

# Train the arithmetic operation classifier (writes data/op_classifier.npz)
python3 scripts/train_op_classifier.py

# With zero-shot semantic operation detection
pip install -e ".[nlp,semantic]"

# Full development environment
pip install -e ".[dev]"
```

### NLP Pipeline Architecture

The reasoning layer uses **zero-hardcoded-vocabulary** NLP.  All linguistic
knowledge lives in pre-trained models — not in word lists or regex.

| Sub-system | Implementation | Replaces |
|---|---|---|
| Entity extraction | spaCy POS (`PROPN`, `NOUN+cap`, `NUM`) | 65-word `_NON_ENTITIES` frozenset |
| Predicate normalisation | spaCy `token.lemma_` + stop-word filter | 49-word `_PROP_STOP` + manual stemmers |
| Auxiliary verb detection | spaCy `token.pos_ == "AUX"` | 13-word `_AUX_VERBS` frozenset |
| Negation exclusion | spaCy `dep_ == "neg"` | 4 hardcoded negation words in `_prop_key` |
| Word-to-number | `word2number` library | 32-entry `_WORD_NUMS` dict |
| Arithmetic operation detection | TF-IDF + LogReg classifier (`_clf.py`) + structural dep-tree | 32-word `_OP_LEMMAS` dict + 76 inline keyword strings |
| NL → Z3 sentence parsing | spaCy dependency tree walk | 45 regex patterns |

### Reasoning Modules

```
pureason/reasoning/
├── _z3utils.py    spaCy NLP utilities (lemma, pred key, entity extraction)
├── _z3ctx.py      Z3 variable registry + dep-tree NL→Z3 parser
├── _clf.py        TF-IDF + LogReg operation classifier (pure-numpy inference)
├── arithmetic.py  Word-problem solver (classifier + structural dep-tree + word2number)
├── syllogism.py   Formal syllogism checker (Z3 + heuristic fallback)
├── chain.py       Chain-of-thought step verifier
├── repair.py      Arithmetic error detector/repairer + majority vote
└── mcq.py         Multiple-choice question evaluator
```

### Verify a Syllogism

```python
from pureason.reasoning import verify_syllogism

report = verify_syllogism(
    premises=["All mammals are warm-blooded.", "Whales are mammals."],
    conclusion="Whales are warm-blooded.",
)
print(report.is_valid)          # True
print(report.chain_confidence)  # 0.88
```

### Solve an Arithmetic Word Problem

```python
from pureason.reasoning import solve_arithmetic

report = solve_arithmetic("Maria earned 50 dollars and spent 23 dollars. How much does she have?")
print(report.answer)  # "27"
print(report.is_valid)
```

---



```bash
git clone https://github.com/sorunokoe/PureReason
cd PureReason
cargo install --path crates/pure-reason-cli --locked
pure-reason calibrate "The patient must have cancer."
```

```bash
# API mode (local, offline)
cargo run -p pure-reason-api -- --bind 127.0.0.1:3000 --allow-unauthenticated

# API with outbound webhooks enabled (explicit opt-in)
cargo run -p pure-reason-api --features webhooks -- --bind 127.0.0.1:3000
```

```bash
# Python wrapper
python -m pip install -e .
python -m pureason._cli calibrate "The stock will definitely double."
```

### Leakage audit (run before trusting any F1)

```bash
python3 benchmarks/benchmark_leak_audit.py --verbose
```

Fails the build if any signal string in `world_priors.rs` overlaps a
benchmark test file. Required to pass before any new prior can land.

---

## References (expanded docs and implementation)

| Topic | Reference |
|---|---|
| Architecture Decision Records | [`docs/ADR-001.md`](./docs/ADR-001.md) (Scale 1 governance), [`docs/ADR-002.md`](./docs/ADR-002.md) (Agentic assurance roadmap) |
| **TRIZ implementation guide** | [`docs/TRIZ-IMPLEMENTATION.md`](./docs/TRIZ-IMPLEMENTATION.md) — Systematic improvements, deployment, performance |
| TRIZ integrity report (2026-04) | [`docs/TRIZ-42.md`](./docs/TRIZ-42.md) |
| Benchmarks and methodology | [`docs/BENCHMARK.md`](./docs/BENCHMARK.md) |
| Methodology and ECS rationale | [`docs/METHODOLOGY.md`](./docs/METHODOLOGY.md) |
| Reproducibility: hashes, seeds, holdout | [`docs/REPRODUCIBILITY.md`](./docs/REPRODUCIBILITY.md) |
| Capability matrix | [`docs/CAPABILITIES.md`](./docs/CAPABILITIES.md) |
| MCP Integration | [`docs/MCP-INTEGRATION.md`](./docs/MCP-INTEGRATION.md) — Agent integration guide |
| CLI commands and output formats | [`crates/pure-reason-cli/`](./crates/pure-reason-cli/) |
| Core Rust engine | [`crates/pure-reason-core/`](./crates/pure-reason-core/) |
| REST API server | [`crates/pure-reason-api/`](./crates/pure-reason-api/) |
| Python wrapper/SDK | [`crates/pure-reason-py/`](./crates/pure-reason-py/) |
| Trust dashboard | [`crates/pure-reason-dashboard/`](./crates/pure-reason-dashboard/) |
| Benchmark runner scripts | [`benchmarks/`](./benchmarks/) |
| Contribution guide | [`.github/CONTRIBUTING.md`](./.github/CONTRIBUTING.md) |

---

## License

Apache 2.0 — see [`LICENSE`](./LICENSE).
