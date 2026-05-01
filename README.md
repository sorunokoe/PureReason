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
[![Tests](https://img.shields.io/badge/tests-618%20passing-brightgreen.svg)](#)
[![MCP](https://img.shields.io/badge/integration-MCP%2FCLI-blue.svg)](#quick-start)

**Fast hallucination detection for AI systems**

</div>

---

## What is PureReason?

**PureReason** verifies AI model outputs for hallucinations, contradictions, and overconfidence. It's a **verification layer** that works alongside frontier models (GPT, Claude, Gemini) - not a replacement for them.

**Use it when you need:**
- ✅ Fast verification (<5ms per check)
- ✅ Hallucination detection
- ✅ Explainable decisions
- ✅ Offline operation (zero API costs)
- ✅ Safety layer for AI agents

**Don't use it for:**
- ❌ General reasoning (use GPT-5, Claude, o1)
- ❌ Problem solving (it verifies, doesn't generate)
- ❌ Content generation

## Benchmarks

PureReason achieves strong performance on hallucination detection benchmarks:

| Benchmark | F1 Score | Task |
|-----------|----------|------|
| **HaluEval QA** | **0.871** | Question answering verification |
| **LogicBench** | **0.846** | Structural logic detection |
| **TruthfulQA** | **0.798** | Misconception detection |
| **HalluLens** | **0.729** | Grounding + contradiction checks |
| **FELM** | 0.645 | Segment-level factuality |
| **RAGTruth** | 0.646 | Grounded hallucination detection |
| **HalluMix** | 0.664 | Multi-domain hallucination |
| **HaluEval Dialogue** | 0.634 | Dialogue verification |
| **FaithBench** | 0.622 | Summarization faithfulness |

**Performance gains** (v0.3.1):
- +25-30pp F1 improvement over baseline
- -40% latency reduction
- ±5pp ECS accuracy (vs ±15pp drift before)

**Full methodology**: See [`docs/BENCHMARK.md`](./docs/BENCHMARK.md) and [`docs/REPRODUCIBILITY.md`](./docs/REPRODUCIBILITY.md)

## How It Works

```text
Input:  "The patient must have cancer."
Output: Risk: HIGH | Confidence: 34/100
Flag:   Certainty overreach
Rewrite:"The patient has findings consistent with possible malignancy."
```

PureReason combines:
- **Symbolic logic** - Deterministic verification using Z3
- **Neural embeddings** - Semantic similarity detection (all-MiniLM-L6-v2)
- **Domain calibration** - Per-domain accuracy tuning
- **Knowledge grounding** - Entity checking and contradiction detection

The typical workflow:
1. **Frontier model** (GPT, Claude) generates output
2. **PureReason** verifies and scores it (0-100 ECS)
3. Agent receives verification + regulated text
4. High-risk outputs flagged for human review

## Quick Start

### 1. Standalone CLI

```bash
cargo install --path crates/pure-reason-cli --locked
pure-reason review "The patient must have cancer."
```

### 2. MCP Integration (for AI agents)

```bash
# Build the MCP server
cargo build --release -p pure-reason-mcp

# Add to your agent's MCP config
# Full guide: docs/MCP-INTEGRATION.md
```

Your agent (Claude Desktop, Cursor, GitHub Copilot) can then call PureReason verification tools.

### 3. Python API

```bash
pip install pureason[semantic,logic,nlp]
```

```python
from pureason import verify

result = verify("Aspirin cures all cancers.")
print(result["risk_level"])  # HIGH
print(result["has_illusions"])  # True
```

### 4. REST API

```bash
cargo run -p pure-reason-api -- --bind 127.0.0.1:3000
```

## Core Features

- **Hallucination detection** - Catches contradictions, fabrications, entity errors
- **Confidence scoring** - 0-100 ECS with domain-aware calibration  
- **Reasoning verification** - Chain-of-thought and arithmetic step checking
- **Text regulation** - Rewrites overconfident claims to hedged language
- **Multiple interfaces** - CLI, MCP, Python, Rust library, REST API
- **Offline operation** - No API keys required, runs completely local
- **Explainable results** - Traceable verification logic with evidence

## Example: Chain-of-Thought Verification

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

PureReason verifies each step deterministically and pinpoints exact failures.

## Advanced Usage

### Python Reasoning Layer

```python
# Verify formal syllogisms
from pureason.reasoning import verify_syllogism

report = verify_syllogism(
    premises=["All mammals are warm-blooded.", "Whales are mammals."],
    conclusion="Whales are warm-blooded.",
)
print(report.is_valid)  # True

# Solve arithmetic word problems
from pureason.reasoning import solve_arithmetic

report = solve_arithmetic("Maria earned 50 dollars and spent 23 dollars. How much?")
print(report.answer)  # "27"
```

### Build from Source

```bash
git clone https://github.com/sorunokoe/PureReason
cd PureReason
cargo build --release
./target/release/pure-reason review "Your text here"
```

---

## Documentation

| Topic | Link |
|-------|------|
| **Benchmarks** | [`docs/BENCHMARK.md`](./docs/BENCHMARK.md) - Full results and methodology |
| **Reproducibility** | [`docs/REPRODUCIBILITY.md`](./docs/REPRODUCIBILITY.md) - Seeds, hashes, holdout |
| **MCP Integration** | [`docs/MCP-INTEGRATION.md`](./docs/MCP-INTEGRATION.md) - Agent setup guide |
| **Capabilities** | [`docs/CAPABILITIES.md`](./docs/CAPABILITIES.md) - Feature matrix |
| **TRIZ Guide** | [`docs/TRIZ-IMPLEMENTATION.md`](./docs/TRIZ-IMPLEMENTATION.md) - Performance improvements |
| **API Reference** | [`crates/pure-reason-core/`](./crates/pure-reason-core/) - Core Rust engine |
| **Contributing** | [`.github/CONTRIBUTING.md`](./.github/CONTRIBUTING.md) - How to contribute |

## Use Cases

**Best for:**
- Verifying AI agent outputs before execution
- Detecting hallucinations in RAG systems
- Scoring confidence in generated claims
- Offline reasoning verification
- Production AI safety layers
- Code agents needing local verification

**Not suitable for:**
- Novel problem solving (use GPT-5, Claude, o1)
- Long-context reasoning (>10K tokens)
- Real-time streaming (optimized for batch)
- Content generation

## License

Apache 2.0 — see [`LICENSE`](./LICENSE)
