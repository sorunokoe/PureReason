# Competitive analysis: evidence-first framing

> This is a positioning document, not a benchmark report. Use it only alongside
> [`CAPABILITIES.md`](./CAPABILITIES.md), [`BENCHMARK.md`](./BENCHMARK.md), and
> [`REPRODUCIBILITY.md`](./REPRODUCIBILITY.md).

## Current competitive position

PureReason's credible position today is **not** “market leader in reasoning.”
Its credible position is:

> **an open-source, deterministic assurance layer with reproducible
> verification benchmarks and a path toward workflow-level review and control.**

## Evidence-backed differentiators

| Dimension | What we can say now | Evidence | Important limit |
|---|---|---|---|
| Determinism | Local kernel paths are deterministic and reproducible | Published seed/hash/holdout workflow | Determinism alone does not prove superior task quality |
| Auditability | Core signals are inspectable and linked to specific heuristics/modules | Capability ledger + source code | Not yet a full reviewer-operations product |
| Deployment control | CLI, API, Python, dashboard, and local deployment surfaces exist | Repo structure and shipped crates | Not proof of enterprise adoption or operability at scale |
| Verification quality | Strong results on selected hallucination/error-detection datasets | `BENCHMARK.md` results on 9 official datasets | Does not establish general reasoning or generation parity |
| Evidence discipline | Published summaries, methodology caveats, and reproducibility protocol exist | `BENCHMARK.md`, `METHODOLOGY.md`, `REPRODUCIBILITY.md` | Current evidence is benchmark-centric, not workflow-outcome-centric |

## Honest comparison frame

### Frontier reasoning models

Frontier models remain stronger candidates for:

- open-ended generation
- broad world knowledge
- long-context synthesis
- flexible reasoning on tasks PureReason has not benchmarked

That means the honest positioning is **complementary**:

- frontier/open/local models generate or plan
- PureReason verifies, calibrates, constrains, and records evidence before
  action

### Evidence and fact-checking systems

These systems overlap more directly with PureReason's verification story, but we
should not claim superiority unless we run **same-task, same-split, same-metric**
comparisons from this repo and publish both outputs.

### Guardrail / evaluation tooling

This is the closest strategic neighborhood today. The differentiation should be:

- deterministic local checks
- reproducible evidence artifacts
- explicit provenance/audit posture
- focus on engineering assurance workflows rather than generic chatbot safety

## Best near-term wedge

The most defensible commercial wedge is:

- AI platform teams
- engineering orgs adopting coding agents
- workflows where plans, tool calls, and changes need verification before merge
  or execution

Concrete starting points:

1. software plan/spec review
2. software change review
3. adjacent engineering assurance tasks where auditability matters

## Claims we can defend

- PureReason is deterministic on its local verification paths.
- PureReason has reproducible benchmark artifacts for its published results.
- PureReason is better framed as an assurance/control layer than as a model
  trying to replace frontier systems.
- PureReason is stronger where transparency, repeatability, and reviewability
  matter more than open-ended generation.

## Claims we should stop making

- best-in-class / leader / winner language
- unmatched speed or cost claims without a published matched-workload method
- regulatory certification or legal/medical deployment safety claims
- head-to-head accuracy claims against o3, DeepSeek, EVICheck, or others unless
  evaluated side by side from this repo
- segment-ownership and market-size claims presented as product proof

## Evidence required before stronger competitive language

1. Matched baseline runs against named competitors on the same benchmark split.
2. Workflow benchmarks for plan/spec review and change review.
3. Published latency and cost measurements with hardware/runtime assumptions.
4. Customer or internal case studies showing review-load reduction, incident
   avoidance, or safer automation.

## Working positioning statement

> **PureReason is an evidence-first assurance layer for agentic engineering
> workflows: deterministic where possible, auditable by design, and intended to
> verify model plans and outputs before action.**
