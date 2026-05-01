# PureReason executive summary

> This is a strategy-facing summary. For source-of-truth evidence, defer to
> [`CAPABILITIES.md`](./CAPABILITIES.md), [`BENCHMARK.md`](./BENCHMARK.md),
> [`METHODOLOGY.md`](./METHODOLOGY.md), and
> [`REPRODUCIBILITY.md`](./REPRODUCIBILITY.md).

## Executive position

PureReason should be described today as an **agentic reasoning assurance
foundation**: a deterministic verification kernel with reproducible benchmark
artifacts and multiple deployment surfaces that can sit beside frontier or
local models.

It should **not** yet be described as a frontier-model replacement, benchmark
leader, or certified solution for regulated industries.

## What is measured today

- Deterministic verification primitives for contradiction, grounding,
  arithmetic, logic, and calibration checks.
- Published benchmark results on **9 official hallucination/error-detection
  datasets** with reproducibility commands, seeds, hashes, and holdout tables.
- Current headline benchmark results from [`BENCHMARK.md`](./BENCHMARK.md):
  - HaluEval QA: **0.871 F1**
  - LogicBench: **0.846 F1**
  - TruthfulQA: **0.798 F1**
  - HalluLens: **0.729 F1**
- Product surfaces already present in-repo: Rust core, CLI, API, Python
  bindings, optional LLM layer, and dashboard.

## What these results do and do not prove

These results support claims about **deterministic verification quality on
binary hallucination/error-detection tasks**.

They do **not** yet support claims that PureReason:

- matches or beats o3, DeepSeek, or other frontier models on general reasoning
- is “best-in-class” overall
- is certified or deployment-proven for medical, legal, or financial decisions
- has published cost/latency superiority on matched workloads
- has demonstrated workflow ROI in production engineering teams

## Product narrative that fits the evidence

The strongest evidence-aligned story is:

- **assurance layer, not frontier-model substitute**
- **verification and auditability first**
- **best suited to workflows that need reproducibility, provenance, and human
  review**
- **initial wedge: software plan/spec review and software change review** for
  AI platform and engineering teams

This keeps PureReason aligned with the direction in
[`ADR-002.md`](./ADR-002.md): complement frontier models by checking plans,
tool calls, and outputs before action.

## Aspirations to keep clearly labeled as roadmap

The following are useful strategic goals, but they are not current evidence
claims:

- model routing and policy gateway
- verifier/reviewer loops around agentic workflows
- evidence and provenance storage
- human operations queues and escalation policies
- matched competitive comparisons on workflow tasks

## Safe claims to use now

- “PureReason provides deterministic verification primitives with reproducible
  benchmark artifacts.”
- “PureReason is being shaped into an agentic reasoning assurance engine for
  engineering workflows.”
- “PureReason complements frontier models by checking plans, outputs, and
  reasoning traces before action.”

## Claims to avoid until new evidence is published

- “most deterministic / most explainable / fastest / cheapest reasoning system”
- “competitive with or better than o3 on general reasoning”
- “production-ready for regulated enterprises”
- “benchmark leadership on BIG-Bench, MMLU-Pro, ARC, GSM8K, HumanEval, DROP,
  or MATH”
- “court-admissible”, “FDA-level”, “compliance-certified”, or similar
  certification language

## Next evidence milestones

1. Publish matched workflow evaluations for plan/spec review and change review.
2. Run head-to-head baselines on the same task, sample, and split before making
   competitive claims.
3. Publish hardware-scoped latency and cost methodology for local and hosted
   deployments.
4. Promote roadmap items only after code, results, and reproducibility
   artifacts exist.
