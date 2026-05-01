# Benchmarking strategy and publication rules

> For current published numbers, use [`BENCHMARK.md`](./BENCHMARK.md). This
> document defines what counts as evidence, what is still backlog, and how new
> benchmark claims should be published.

## Current measured benchmark scope

PureReason currently has published results on **9 official hallucination/error
and verification datasets**. These are primarily **binary detection or
validation tasks**, not frontier-model general reasoning leaderboards.

| Benchmark | Published F1 | Notes |
|---|---:|---|
| HaluEval QA | 0.871 | Strongest grounded QA result |
| LogicBench | 0.846 | Structural logic benchmark; task-mismatch caveat vs external MC systems |
| TruthfulQA | 0.798 | Binary misconception detection, not the generative MC task |
| HalluLens | 0.729 | Grounding novelty + contradiction checks |
| FELM | 0.645 | Segment-level factuality; semantic divergence + arithmetic |
| RAGTruth | 0.646 | Recall-heavy grounded verification |
| HalluMix | 0.664 | Near the current signal ceiling described in `METHODOLOGY.md` |

## What is not yet evidence

The repo contains benchmark expansion scaffolding and competitive-analysis code
paths, but the following should be treated as **unverified until results are
committed and reproducible**:

- BIG-Bench
- MMLU-Pro
- ARC
- GSM8K
- HumanEval
- DROP
- MATH
- synthetic or internally constructed competitive-analysis tasks

In practice, that means estimated F1 ranges, “trajectory” charts, and projected
benchmark wins should stay out of high-level positioning docs until real result
artifacts exist.

## Publication rules

1. **Only cite numbers backed by committed result artifacts.**
   Every headline benchmark claim must map to a committed summary under
   `benchmarks/results/` and to a documented reproduction command.

2. **Label the evidence class.**
   Distinguish standard/calibration-exposed runs from holdout runs as described
   in [`METHODOLOGY.md`](./METHODOLOGY.md).

3. **No external apples-to-oranges comparisons.**
   If a competitor number was not produced on the same task/split/metric from
   this repo, it may be discussed as context but not as proof that PureReason
   wins or loses.

4. **Do not turn benchmark results into market or compliance claims.**
   A benchmark delta does not prove regulatory readiness, workflow ROI, or
   product leadership.

5. **Separate measured capability from benchmark backlog.**
   Planned suites, benchmark harnesses, and experimental modules are roadmap
   material until they produce reproducible outputs.

## Backlog: benchmark expansion with evidence discipline

### 1. Workflow-native evaluation

Priority should shift toward the product direction in [`ADR-002.md`](./ADR-002.md):

- plan/spec review quality
- software change review quality
- escalation precision/recall
- trace completeness and provenance coverage

### 2. Matched competitive baselines

Where competitive claims matter, run baselines on the exact same samples and
publish:

- PureReason output
- baseline output
- evaluation script
- latency/cost assumptions
- caveats about task mismatch if they remain

### 3. Benchmark governance

Before expanding the benchmark story further:

- label official vs synthetic datasets clearly
- document any benchmark-specific heuristics or thresholds
- keep result hashes and holdout summaries current
- avoid using “framework ready” as shorthand for “performance proven”.

## Safe summary for external use

- PureReason has reproducible benchmark evidence on selected verification tasks
  today.
- PureReason has additional benchmark infrastructure, but unrun suites remain
  backlog, not proof.
- New benchmark claims should ship only with artifacts, methodology, and
  reproducibility steps.
