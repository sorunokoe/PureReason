# Benchmark quick reference

> This is the short-form companion to [`BENCHMARK.md`](./BENCHMARK.md),
> [`METHODOLOGY.md`](./METHODOLOGY.md), and
> [`REPRODUCIBILITY.md`](./REPRODUCIBILITY.md). It summarizes **published
> evidence only**.

## Current published benchmark scope

PureReason currently publishes results on **9 official verification /
hallucination-detection datasets**. These are mostly binary detection tasks,
not general reasoning leaderboards.

| Benchmark | F1 | What it shows |
|---|---:|---|
| HaluEval QA | 0.871 | Strongest grounded QA verification result |
| LogicBench | 0.846 | Strong structural logic detection result |
| TruthfulQA | 0.798 | Misconception detection on labelled binary pairs |
| HalluLens | 0.729 | Grounding novelty + contradiction checks |
| FELM | 0.645 | Segment-level factuality with semantic divergence |
| RAGTruth | 0.646 | Recall-heavy grounded hallucination detection |
| HalluMix | 0.664 | Near current signal ceiling documented in methodology |
| HaluEval Dialogue | 0.634 | Dialogue-grounding verification |
| FaithBench | 0.622 | Summarization faithfulness detection |

## What this evidence supports

- deterministic verification claims on the published benchmark formats
- reproducibility claims based on committed summaries, seeds, and hashes
- positioning PureReason as a verification/assurance system rather than a
  frontier model replacement

## What this evidence does not support yet

- benchmark leadership on BIG-Bench, MMLU-Pro, ARC, GSM8K, HumanEval, DROP, or
  MATH
- head-to-head claims against o3, DeepSeek, EVICheck, or other named systems
  unless they are run on the same split from this repo
- generalized claims like “best-in-class”, “fastest”, or “cheapest”
- regulatory certification or workflow ROI claims

## Reproduce the published results

```bash
# Standard published sweep
python3 benchmarks/run_downloaded_benchmarks.py --n 200 --seed 42

# Holdout sweep
python3 benchmarks/run_downloaded_benchmarks.py --n 200 --seed 42 --holdout
```

## Benchmark backlog

These are valid next targets, but they remain backlog until result artifacts are
committed:

- BIG-Bench
- MMLU-Pro
- ARC
- GSM8K
- HumanEval
- DROP
- MATH
- workflow benchmarks for plan/spec review and change review

## Safe external summary

> PureReason has reproducible benchmark evidence on selected deterministic
> verification tasks today, and an evidence-first roadmap for expanding that
> coverage into workflow-level agent assurance.
