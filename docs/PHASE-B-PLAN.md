# Scale 2 Phase B: Distilled Model (DistilBERT Classifier)

**Current Status**: Phase A complete (+0.010 F1), Phase A2 complete (stable)  
**Phase B Goal**: Train binary classifier for +0.090 F1 additional  
**Timeline**: 3-4 weeks  
**Expected F1**: 0.603 → 0.693 (+0.090)

---

## Problem Statement

**Heuristic detectors plateau** at ~0.60 F1 because they miss:

1. **Numeric hallucinations** (FELM: 0.262 F1)
   - Scientific constants falsified
   - Medical dosages incorrect
   - Statistical claims invented
   - Heuristics can't validate without domain DB

2. **Open-world hallucinations** (HalluMix: 0.167 F1)
   - General facts not in training data
   - Plausible-sounding but false claims
   - Too much variance for pattern matching

3. **Ambiguous claims** (TruthfulQA: 0.812 F1)
   - Claims that *could* be true but aren't
   - Subtle factual errors
   - Require understanding of world knowledge

**Solution**: Train a lightweight neural classifier to learn these patterns from data.

---

## Architecture: DistilBERT Binary Classifier

### Model Choice

**Why DistilBERT?**
- 40% smaller than BERT (faster inference, fits on edge)
- Maintains 97% of BERT performance
- 2-3 layers only (not full 12)
- PyTorch pretrained weights available
- Fine-tuning on 2000-3000 examples realistic

### Binary Task

```
Input: [CLS] knowledge [SEP] claim [SEP]
Output: logits for [FALSIFIABLE, UNFALSIFIABLE]

FALSIFIABLE = "This claim can be verified as false"
             = Hallucination risk
             
UNFALSIFIABLE = "This claim is consistent with knowledge"
              = Safe/grounded
```

### Ensemble Integration

```
Phase A heuristic ensemble: confidence_h in [0.0, 1.0]
Phase B model prediction: prob_model in [0.0, 1.0]

Final confidence = 0.7 * confidence_h + 0.3 * prob_model

Rationale:
- Heuristics more interpretable (70% weight)
- Model catches gaps (30% weight)
- Ensemble handles model uncertainty gracefully
```

---

## Training Data

### Data Source 1: FELM Benchmark

**File**: `benchmarks/downloads/felm/all.jsonl`  
**Size**: ~1000 samples (factual + non-factual)  
**Format**: 
```json
{
  "claim": "Planck's constant is 6.626 × 10^-34 J·s",
  "label": 1,  // 1 = factual, 0 = non-factual
  "category": "physics"
}
```

**Weak Label Generation**:
```python
# Use heuristic verdict as weak label
verdict = ensemble.verify(knowledge=None, answer=claim)
weak_label = 1 if verdict.hallucination_probability < 0.5 else 0
```

### Data Source 2: TruthfulQA

**File**: `benchmarks/downloads/truthfulqa/TruthfulQA.csv`  
**Size**: ~800 samples (questions + reference answers)  
**Format**:
```
question,best_answer,correct_answers,incorrect_answers
"What is the capital of Australia?","Canberra","Canberra","Sydney; Melbourne; Brisbane"
```

**Weak Label Generation**:
```python
# Parse reference + incorrect answers
correct = set(parse_answers(best_answer))
incorrect = set(parse_answers(incorrect_answers))

# Create synthetic pairs
for answer in correct:
    create_training_pair(question, answer, label=1)  # UNFALSIFIABLE
    
for answer in incorrect:
    create_training_pair(question, answer, label=0)  # FALSIFIABLE
```

### Data Source 3: HaluEval

**File**: `benchmarks/downloads/halueval/qa_data.json`  
**Size**: ~500 samples (grounded + hallucinated Q&A)

### Total Dataset

```
FELM:     ~1000 samples
TruthfulQA: ~1500 samples (1 correct × 2-3 incorrect = synthetic pairs)
HaluEval:   ~500 samples
────────────────────────
Total:    ~3000 samples

Split:
- Training:   70% (2100 samples)
- Validation: 10% (300 samples)
- Test:       20% (600 samples)
```

---

## Training Procedure

### Step 1: Data Preparation (1-2 days)

```python
# Pseudo-code
def prepare_training_data():
    data = []
    
    # FELM: numeric/scientific claims
    for entry in load_felm():
        pair = format_for_distilbert(
            knowledge="",  # FELM has no context
            claim=entry["claim"]
        )
        label = 1 if entry["label"] == "factual" else 0
        data.append((pair, label))
    
    # TruthfulQA: factual Q&A
    for question, answers in load_truthfulqa():
        for answer in answers["correct"]:
            pair = format_for_distilbert(
                knowledge=f"Question: {question}",
                claim=answer
            )
            data.append((pair, 1))
        
        for answer in answers["incorrect"]:
            pair = format_for_distilbert(
                knowledge=f"Question: {question}",
                claim=answer
            )
            data.append((pair, 0))
    
    # Balance classes
    balanced = balance_classes(data)  # Oversample minority
    
    # Split
    train, val, test = split_80_10_10(balanced)
    
    return train, val, test
```

### Step 2: Fine-tuning (2-3 days)

```python
# Architecture
model = DistilBertForSequenceClassification.from_pretrained(
    'distilbert-base-uncased',
    num_labels=2
)

# Hyperparameters
epochs = 5
batch_size = 32  # Fits in memory
learning_rate = 2e-5
optimizer = AdamW(model.parameters(), lr=learning_rate)

# Training loop
for epoch in range(epochs):
    for batch in train_loader:
        outputs = model(**batch)
        loss = outputs.loss
        loss.backward()
        optimizer.step()
    
    # Validation
    val_f1 = evaluate_on_validation_set()
    if val_f1 > best_f1:
        save_checkpoint()
        best_f1 = val_f1
```

### Step 3: Evaluation (1 day)

```python
# Test on held-out set
test_pred = model.predict(test_data)
test_f1 = compute_f1(test_pred, test_labels)

# Per-benchmark evaluation
for benchmark in ["FELM", "TruthfulQA", "HaluEval"]:
    benchmark_f1 = evaluate_on_benchmark(benchmark)
    print(f"{benchmark}: F1 = {benchmark_f1:.3f}")

# Check for overfitting
if val_f1 - test_f1 > 0.10:
    # Reduce complexity or add dropout
    reduce_model_size()
```

---

## Expected Results

### Benchmark Improvements

| Benchmark | Before B | After B | Delta | Reason |
|-----------|----------|---------|-------|--------|
| **FELM** | 0.262 | 0.450 | +0.188 | Numeric patterns learned |
| **HalluMix** | 0.167 | 0.420 | +0.253 | General hallucination patterns |
| **TruthfulQA** | 0.812 | 0.850 | +0.038 | Open-world ambiguity handling |
| **HalluEval Dialogue** | 0.602 | 0.650 | +0.048 | Conversation patterns |
| **HalluLens** | 0.762 | 0.780 | +0.018 | Maintains performance |
| **LogicBench** | 0.821 | 0.835 | +0.014 | Slight improvement |
| **Average** | **0.603** | **0.693** | **+0.090** | **Significant gain** |

### Why These Improvements?

1. **FELM +0.188**: Model learns numeric/scientific patterns (what's plausible)
2. **HalluMix +0.253**: Catches general hallucinations heuristics miss
3. **TruthfulQA +0.038**: Better at subtle ambiguity
4. **No regression**: 30% weight from model, 70% from heuristics

---

## Implementation Checklist

- [ ] Step 1: Data preparation
  - [ ] Load and parse FELM, TruthfulQA, HaluEval
  - [ ] Generate weak labels from heuristic ensemble
  - [ ] Create synthetic pairs (Q&A)
  - [ ] Balance classes
  - [ ] Split train/val/test
  
- [ ] Step 2: Model training
  - [ ] Load DistilBERT pretrained
  - [ ] Create training loop
  - [ ] Monitor validation F1
  - [ ] Save best checkpoint
  
- [ ] Step 3: Evaluation
  - [ ] Test on held-out set
  - [ ] Per-benchmark evaluation
  - [ ] Check for overfitting
  - [ ] Analyze error cases
  
- [ ] Step 4: Integration
  - [ ] Serialize model to ONNX (optional, for speed)
  - [ ] Create Python wrapper for prediction
  - [ ] Integrate into pipeline (70/30 weighting)
  - [ ] Add to reproducibility docs
  
- [ ] Step 5: Validation
  - [ ] Run full benchmark suite
  - [ ] Validate +0.090 F1 achieved
  - [ ] No regression on Phase A benchmarks
  - [ ] Document results in PHASE-B-RESULTS.md

---

## Code Changes Summary

**Files to create**:
- `scripts/train_distilbert.py` (training script, ~150 LOC)
- `scripts/distilbert_eval.py` (evaluation script, ~100 LOC)
- `crates/pure-reason-py/src/lib.rs` (Python model wrapper, ~50 LOC PyO3)
- `docs/PHASE-B-RESULTS.md` (results summary)

**Files to modify**:
- `crates/pure-reason-core/src/pipeline.rs` (+20 LOC for 70/30 weighting)
- `benchmarks/run_downloaded_benchmarks.py` (+30 LOC for model integration)

**Total new LOC**: ~200-250 lines

---

## Risk Assessment

### Risk 1: Overfitting

**Likelihood**: Medium  
**Impact**: Model memorizes training data, poor generalization  
**Mitigation**:
- Use early stopping (validation F1)
- Limit to 5 epochs max
- 10% dropout rate
- L2 regularization

### Risk 2: Weak Labels Noisy

**Likelihood**: Medium  
**Impact**: Model trained on incorrect labels  
**Mitigation**:
- Validate heuristic labels on clean test set
- Use label smoothing (0.1)
- Manual review of hard examples

### Risk 3: Benchmark Leakage

**Likelihood**: Low (data split carefully)  
**Impact**: Test F1 inflated  
**Mitigation**:
- Use only held-out splits for training
- Strict separation of train/val/test
- Audit for overlap with benchmark test sets

### Risk 4: Model Too Large

**Likelihood**: Very low (DistilBERT tiny)  
**Impact**: Can't deploy or too slow  
**Mitigation**:
- DistilBERT inherently small
- Can prune further if needed
- ONNX export for inference speedup

---

## Success Criteria

- [x] Phase A2 stable (baseline maintained)
- [ ] Training data prepared (3000 samples, balanced)
- [ ] Model trained (5 epochs, validation F1 > 0.85)
- [ ] Evaluation complete
  - [ ] Test F1 > 0.80 (no overfitting)
  - [ ] FELM +0.15+ F1 improvement
  - [ ] HalluMix +0.20+ F1 improvement
  - [ ] Average +0.080-0.100 F1 improvement
- [ ] Integrated into pipeline
  - [ ] 70/30 weighting applied
  - [ ] No breaking changes
  - [ ] Full benchmark suite passing
- [ ] Documented and committed

---

## Timeline Estimate

- **Data prep**: 2 days (parsing, weak labeling, splitting)
- **Training**: 2-3 days (fine-tuning, hyperparameter tuning)
- **Evaluation**: 1 day (test set, per-benchmark analysis)
- **Integration**: 1-2 days (pipeline changes, testing)
- **Documentation**: 1 day (results, lessons learned)

**Total**: 7-10 days of work (3-4 calendar weeks with reviews/breaks)

---

## Decision Points

### After Training

**Decision 1**: If test F1 < 0.80, should we:
- A) Accept overfitting trade-off (use model anyway)
- B) Reduce model complexity (smaller DistilBERT)
- C) Add more training data
- D) Skip Phase B, do Phase A3 (domain validation) instead

**Decision 2**: If average F1 only +0.050 (below +0.080 target):
- A) Accept smaller gain, deploy anyway
- B) Combine with Phase A3 (domain validation)
- C) Skip Phase B, focus on other improvements

### Recommendation

**Proceed if**: 
- Test F1 > 0.80 AND
- Average F1 > 0.650 (baseline +0.047)

**Proceed with caution if**:
- Test F1 = 0.75-0.80 (light overfitting acceptable)
- OR Average F1 = 0.640-0.650 (smaller gain but worth it)

**Don't proceed if**:
- Test F1 < 0.70 (heavy overfitting, poor generalization)
- OR Average F1 < 0.610 (actually worse than baseline)

---

## References

- **Phase A**: `docs/SCALE2-PHASE-A.md`
- **Phase A2**: `docs/PHASE-A2-PLAN.md`
- **Roadmap**: `docs/SCALE2-ROADMAP.md`
- **Benchmarks**: `benchmarks/run_downloaded_benchmarks.py`
- **ADR-002**: Architecture decisions

