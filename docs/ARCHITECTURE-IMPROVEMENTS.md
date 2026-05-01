# Architecture improvements: implemented primitives vs roadmap

> Previous versions of this document treated every new module as a proven F1
> uplift and product win. This reset separates **implemented code** from
> **measured evidence** and **forward-looking architecture**.

## What is implemented and already grounded in evidence

PureReason has a real verification core with measurable outputs in the evidence
ledger and benchmark docs. The strongest validated areas today are:

- contradiction / grounding checks
- misconception and world-prior checks
- arithmetic and numeric plausibility verification
- logic-structure verification
- calibration-oriented scoring and reproducibility tooling

These capabilities are documented in [`CAPABILITIES.md`](./CAPABILITIES.md) and
[`BENCHMARK.md`](./BENCHMARK.md). This document should not be used to introduce
new performance claims that are absent there.

## Implemented components that should not be marketed as proven wins yet

The following modules exist in the codebase and may be useful building blocks,
but their product impact is **not yet independently established** by published
workflow evidence or per-module benchmark studies.

| Area | In repo | What we know | What we do not know yet |
|---|---|---|---|
| Domain routing/tuning (`domain_config`, `confidence_thresholding`, `phase_optimizer`) | Yes | Configuration and calibration helpers exist | No published proof that they create durable gains on target workflows |
| Extended reasoning helpers (`multi_hop_reasoner`, `math_solver`, `error_analyzer`, `uncertainty_calibration`) | Yes | Components are implemented and tested | No defensible repo-wide F1 “trajectory” can be attributed to them as a bundle |
| Learning loop pieces (`meta_learner`, `human_feedback`) | Yes | Early infrastructure exists | No published evidence yet that these loops improve real user outcomes |
| Benchmark/competitive scaffolding (`benchmark_integration`, `specialized_benchmarks`, `competitive_analysis`) | Yes | Harnesses and report structures exist | Harness readiness is not the same as validated benchmark performance |

## Architecture direction that still makes strategic sense

The repo's strongest long-term path remains the one captured in
[`ADR-002.md`](./ADR-002.md): move from a benchmark-centric verifier toward an
**agentic reasoning assurance engine**.

The most important architecture layers are:

1. **Model and policy gateway** — route to frontier, open, or local models with
   explicit budgets and policies.
2. **Verifier/runtime layer** — inspect plans, tool calls, and outputs before
   execution.
3. **Evidence and provenance layer** — persist traces, supporting evidence, and
   replay/debug artifacts.
4. **Human operations layer** — review queues, escalation paths, and audit logs.
5. **Learning loop** — use reviewer feedback and failures to improve routing,
   policies, and specialist checks.

## Promotion policy: when a component becomes a product claim

A module or architectural layer should move from “roadmap/infrastructure” to
“capability claim” only when all of the following are true:

- it is active in a user-facing workflow
- the workflow or benchmark result is published
- reproduction steps and artifacts exist
- latency/cost tradeoffs are measured
- caveats are documented in the evidence docs

## Recommended narrative

Use this document to explain **where the system is going**, not to claim that
all intermediate components are already validated.

A safe summary is:

> PureReason already has meaningful deterministic verification primitives. The
> next architecture step is to wrap those primitives around agentic engineering
> workflows with explicit evidence, provenance, and human review.
