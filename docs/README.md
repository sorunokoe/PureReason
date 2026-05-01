# PureReason documentation index

PureReason now has two kinds of documentation:

1. **evidence-first docs** that describe what is actually measured or implemented
2. **strategy docs** that describe where the product is going

If documents disagree, trust the following order:

1. code
2. [`CAPABILITIES.md`](./CAPABILITIES.md)
3. [`BENCHMARK.md`](./BENCHMARK.md)
4. [`REPRODUCIBILITY.md`](./REPRODUCIBILITY.md)
5. roadmap/strategy docs

---

## Start here

1. [`../README.md`](../README.md) — current product framing
2. [`CAPABILITIES.md`](./CAPABILITIES.md) — measured capabilities and caveats
3. [`BENCHMARK.md`](./BENCHMARK.md) — current benchmark tables
4. [`REPRODUCIBILITY.md`](./REPRODUCIBILITY.md) — how to validate published numbers
5. [`ADR-002.md`](./ADR-002.md) — agentic reasoning assurance roadmap

---

## Evidence-first documents

| Document | Use it for |
|---|---|
| [`CAPABILITIES.md`](./CAPABILITIES.md) | What PureReason can actually do today |
| [`BENCHMARK.md`](./BENCHMARK.md) | Benchmark tables and known gaps |
| [`METHODOLOGY.md`](./METHODOLOGY.md) | Metric conventions and evaluation caveats |
| [`REPRODUCIBILITY.md`](./REPRODUCIBILITY.md) | Hashes, seeds, and validation workflow |
| [`ADR-001.md`](./ADR-001.md) | Scale 1 governance constraints |
| [`TRIZ-IMPLEMENTATION.md`](./TRIZ-IMPLEMENTATION.md) | TRIZ improvements: deployment, performance, validation |

---

## Strategy documents

| Document | Use it for |
|---|---|
| [`ADR-002.md`](./ADR-002.md) | Long-term direction in the agentic era |
| [`EXECUTIVE-SUMMARY.md`](./EXECUTIVE-SUMMARY.md) | High-level positioning draft |
| [`COMPETITIVE-ANALYSIS.md`](./COMPETITIVE-ANALYSIS.md) | Market framing and comparison hypotheses |
| [`BENCHMARKING.md`](./BENCHMARKING.md) | Benchmark expansion strategy and targets |
| [`ARCHITECTURE-IMPROVEMENTS.md`](./ARCHITECTURE-IMPROVEMENTS.md) | Forward-looking architecture ideas |
| [`TRIZ-42.md`](./TRIZ-42.md) | TRIZ analysis and improvement roadmap |

These files are useful for direction-setting, but they should not override the
evidence-first documents when making capability claims.

---

## Current product direction

PureReason is moving toward an **agentic reasoning assurance** role:

- complement frontier models rather than replace them
- let existing frontier agents call PureReason through MCP, CLI, or loopback-local services
- verify plans, tool calls, and outputs before action
- add memory, evidence, provenance, and auditability
- support human review and escalation paths
- keep the default local workflow usable without provider credentials
- start with **software plan/spec review** and **software change review** for
  AI platform and engineering teams

By default, local MCP/CLI state now lives under `~/.pure-reason/agent-state/`
as SQLite task, trace, and review-evidence stores. Set `PURE_REASON_STATE_DIR`
to move that state elsewhere, and set it explicitly on systems where no home
directory can be resolved.

---

## Existing product surfaces

- Rust core: `crates/pure-reason-core/`
- CLI: `crates/pure-reason-cli/`
- MCP server: `crates/pure-reason-mcp/`
- REST API: `crates/pure-reason-api/`
- Python wrapper: `crates/pure-reason-py/`
- Dashboard: `crates/pure-reason-dashboard/`
- Benchmarks: `benchmarks/`

## Implementation documentation

- [`TRIZ-IMPLEMENTATION.md`](./TRIZ-IMPLEMENTATION.md) — Comprehensive TRIZ deployment guide
- [`meta-learner-v2-design.md`](./meta-learner-v2-design.md) — Session meta-learner architecture
- [`domain-calibration-design.md`](./domain-calibration-design.md) — Domain calibration specification
- [`wikipedia-corpus-schema.md`](./wikipedia-corpus-schema.md) — Corpus format and processing
- [`MCP-INTEGRATION.md`](./MCP-INTEGRATION.md) — Agent integration guide

---

## Notes

- Some strategy documents in this folder were created during benchmark and
  positioning experiments. Keep them, but treat them as secondary to measured
  capability/evidence docs.
- The repo already has meaningful verification primitives; the roadmap is about
  turning those primitives into a trustworthy control layer for real workflows.
