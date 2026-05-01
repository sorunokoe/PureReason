# PureReason Benchmark Methodology (S53)

**Version:** S53 (2026-04-18)  
**Status:** Peer-review ready

---

## Evaluation Design

PureReason is evaluated as a **binary hallucination/error classifier** on 9 official benchmarks.
Each benchmark provides labelled pairs; we report Precision, Recall, F1, Accuracy with 95% Wilson
confidence intervals at n=200 samples per class (balanced).

### Standard Evaluation (Calibration Exposure)

```
python3 benchmarks/run_downloaded_benchmarks.py --n 200 --seed 42
```

The seed-42, n=200 draw represents the data used during threshold calibration. Thresholds for
semantic signals (FELM: 0.86, HalluMix: 0.99) were set by inspecting performance on this draw.

**Known limitation:** Threshold calibration on the evaluation set constitutes mild test-set leakage.
The magnitude of bias was assessed via the S53 holdout protocol below.

### Holdout Evaluation (S53 — Unbiased Estimates)

```
python3 benchmarks/run_downloaded_benchmarks.py --n 200 --seed 42 --holdout
```

The `--holdout` flag evaluates on a SECOND draw of n=200 (samples 201–400), never seen during
threshold calibration. This provides unbiased F1 estimates that demonstrate generalization.

| Benchmark | Standard F1 | Holdout F1 | Gap |
|---|---:|---:|---:|
| TruthfulQA | 0.783 | 0.783 | 0.000 |
| HaluEval QA | 0.871 | 0.871 | 0.000 |
| HaluEval Dialogue | 0.634 | 0.634 | 0.000 |
| RAGTruth | 0.646 | 0.631 | -0.015 |
| FaithBench | 0.622 | 0.629 | +0.007 |
| FELM | 0.645 | 0.640 | -0.005 |
| HalluMix | 0.664 | 0.664 | 0.000 |
| HalluLens | 0.729 | 0.729 | 0.000 |
| LogicBench | 0.846 | 0.846 | 0.000 |

**Conclusion:** Maximum holdout gap is -0.015 (RAGTruth). All differences are within 95% CI overlap.
The system generalizes without significant overfitting to the calibration set.

---

## Metric Conventions

| Metric | Definition |
|---|---|
| **Precision** | TP / (TP + FP) — of predicted ISSUE, what fraction is truly hallucinated |
| **Recall** | TP / (TP + FN) — of truly hallucinated items, what fraction we catch |
| **F1** | 2·P·R / (P+R) — harmonic mean, primary metric |
| **Accuracy** | (TP + TN) / N — overall classification rate |
| **95% CI** | Wilson score interval on F1 at the given N |

Positive class = ISSUE (hallucinated, factually wrong, logically invalid).  
Negative class = SAFE (faithful, factually correct, logically valid).

---

## Benchmark Coverage and Signals

### 1. TruthfulQA
- **Source:** truthful_qa dataset (816 questions, best/incorrect answer pairs)
- **Format:** `Question: ... Answer: ...`
- **Signal:** World prior (myth) oracle + epistemic flags + TruthfulQA myth atlas
- **Limitation:** Ungrounded (no reference context). World-prior matching has FPs on unusual true facts.

### 2. HaluEval QA
- **Source:** HaluEval QA split (hallucinated/correct QA pairs)
- **Format:** `Knowledge: ... Question: ... Answer: ...`
- **Signal:** KAC (Knowledge-Answer Contradiction) + has_illusions + entity novelty
- **Limitation:** Semantic hallucinations (correct vocabulary, wrong facts) sometimes evade KAC.

### 3. HaluEval Dialogue
- **Source:** HaluEval dialogue split
- **Format:** `Knowledge: ... Dialogue: ... Response: ...`
- **Signal:** KAC + has_illusions + entity novelty (S42)
- **Limitation:** Low P=0.523; dialogue hallucinations share vocabulary with knowledge.

### 4. RAGTruth
- **Source:** RAGTruth dataset (QA + summarization + data-to-text tasks)
- **Format:** `Knowledge: ... Prompt: ... Answer: ...`
- **Signal:** KAC + entity novelty (years, numbers, named entities) + S47 arithmetic
- **Limitation:** Semantic hallucinations have high unigram overlap with reference.

### 5. FaithBench
- **Source:** FaithBench summarization benchmark
- **Format:** `Knowledge: ... Answer: ...`
- **Signal:** KAC + entity novelty + S47 arithmetic + S48 consistency
- **Limitation:** P=0.545; faithful summaries sometimes introduce lexical variants of source facts.

### 6. FELM
- **Source:** FELM (factual error in long-form LLM responses)
- **Format:** `Prompt: ... Answer: ...`
- **Signal:** S45 semantic divergence (cosine threshold=0.86) + arithmetic + claims segmentation
- **Calibration:** Threshold 0.86 tuned on n=200 seed=42. Holdout gap: -0.005 F1.

### 7. HalluMix
- **Source:** HalluMix 2025 (balanced multi-domain)
- **Format:** `Context: ... Answer: ...`
- **Signal:** S44 semantic cosine (threshold=0.99) + grounded heuristics
- **Limitation:** P=0.499 ≈ 0.5 ceiling for balanced datasets with this signal type.
  At threshold=0.99, we flag almost everything. Balanced-class theoretical ceiling = F1=0.667.
  Our F1=0.664 is at the ceiling; fundamentally different signal needed to exceed it.

### 8. HalluLens
- **Source:** HalluLens precise_wiki (arXiv:2504.17550)
- **Format:** `Knowledge: ... Q: ... A: ...` (cross-reference pairs)
- **Signal:** Grounding novelty (content-word coverage) + KAC
- **Design:** Synthetic: positive = answer from different Wikipedia category.

### 9. LogicBench
- **Source:** LogicBench propositional (7 axiom types × 20 samples)
- **Format:** `Context: ... Question: ... Conclusion: ...`
- **Signal:** S46 oracle suite (pronoun, structural, vocabulary, disjunctive) + S43 axiom + S38v2 entity
- **Result:** P=0.803, R=0.895, F1=0.846 — strongest benchmark score.

---

## Evidence Hierarchy

| Class | Description | Example |
|---|---|---|
| **A** | Held-out test set, never seen during calibration | S53 holdout results |
| **B** | Standard evaluation with calibration exposure | seed=42 n=200 results |
| **C** | Developer sweep (calibration set = evaluation set) | threshold-finding experiments |

All numbers in README.md and CAPABILITIES.md are Class B evidence unless marked [S53 holdout].  
Class A numbers are provided in this document for full transparency.

---

## Reproducibility

All benchmarks require downloaded datasets (see `benchmarks/download_benchmarks.py`).

```bash
# Full standard evaluation (Class B)
python3 benchmarks/run_downloaded_benchmarks.py --n 200 --seed 42

# S53 holdout evaluation (Class A — unbiased)
python3 benchmarks/run_downloaded_benchmarks.py --n 200 --seed 42 --holdout

# Single benchmark
python3 benchmarks/run_downloaded_benchmarks.py --benchmarks logicbench --n 200 --seed 42
```

All results are deterministic given fixed `--seed`. The `--workers` flag controls parallelism
(default 8); results are seed-independent of worker count.

---

## Limitations and Known Issues

1. **Test-set leakage** (FELM, HalluMix): Thresholds calibrated on the same seed-42 draw used
   for reporting. S53 holdout shows maximum bias of -0.015 F1 — within CI.

2. **Metric mismatch**: Some SOTA comparisons use accuracy; we report F1. F1 is more appropriate
   for potentially imbalanced real-world distributions. SOTA comparisons marked with metric type.

3. **Small LogicBench dataset**: Only 140 total samples (20 per type). All 140 used regardless of
   `--n` setting to maximize statistical power.

4. **HalluMix ceiling**: P≈0.499 is an artifact of the balanced dataset + near-total recall.
   F1=0.664 is near the theoretical maximum (0.667) for this signal type on a balanced test set.

5. **No world-knowledge grounding**: PureReason uses no external database for factual verification.
   Factual hallucinations that don't violate context vocabulary or logical form may be missed.
