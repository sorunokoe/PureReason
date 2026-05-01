# Scale 2 Phase A2: Enhanced Semantic Detector

**Current Status**: Phase A complete (+0.010 F1 average)  
**Next Step**: Implement semantic similarity via spaCy vectors  
**Timeline**: 1 week (3-5 days active development)  
**Expected Impact**: +0.02-0.03 F1 additional

---

## Problem Analysis

Current detectors catch **discrete patterns** (entities, conditionals, elaboration), but miss **semantic drift** — when LLMs add plausible-sounding information that's contextually unrelated.

Example:
```
Knowledge: "Marie Curie won a Nobel Prize"
LLM Output: "Marie Curie won a Nobel Prize, and her favorite hobby was painting"

Current detectors:
- SemanticDriftDetector: ✓ Flags (elaboration detected)
- NoveltyDetector: ✗ Misses (no new proper nouns added)
- FormalLogicChecker: ✗ Not applicable
- NumericDomainDetector: ✗ Not applicable

Problem: LLM added completely plausible but unsupported info ("painting hobby")
Current approach catches via elaboration length, but not semantic coherence
```

**Goal**: Add semantic similarity check to detect when answer **diverges semantically** from knowledge while maintaining elaboration pattern.

---

## Solution: Semantic Similarity Detector

### Algorithm

```python
# Pseudo-code (will be Rust + PyO3 calling spaCy)

def check_semantic_drift(knowledge: str, answer: str) -> DetectorVote:
    # Step 1: Get average word vectors for both texts
    knowledge_doc = spacy(knowledge)
    answer_doc = spacy(answer)
    
    knowledge_vec = average(knowledge_doc.word_vectors)
    answer_vec = average(answer_doc.word_vectors)
    
    # Step 2: Compute cosine similarity
    cosine_sim = dot(knowledge_vec, answer_vec) / (norm(knowledge_vec) * norm(answer_vec))
    
    # Step 3: Flag if low similarity + high elaboration
    is_elaborate = len(answer) > 1.3 * len(knowledge)
    is_divergent = cosine_sim < 0.65  # threshold: tuned per benchmark
    
    if is_divergent and is_elaborate:
        # Semantic drift: answer elaborates but diverges
        confidence = 0.75
        flags_risk = True
        evidence = f"Semantic similarity {cosine_sim:.2f} + elaboration"
    elif is_divergent:
        # Just divergent (could be paraphrase)
        confidence = 0.55
        flags_risk = False
    else:
        # Semantically coherent
        confidence = 0.6
        flags_risk = False
    
    return DetectorVote {
        detector_name: "SemanticSimilarityDetector",
        confidence,
        flags_risk,
        evidence,
    }
```

### Implementation Plan

1. **Add new detector** in `ensemble_verifier.rs` (80 LOC)
   - Name: `SemanticSimilarityDetector`
   - Integrate with spaCy via existing Python bridge
   - Add to `EnsembleVerifier::verify()` vote collection

2. **Update pipeline** in `pipeline.rs`
   - Call new detector in `compose_verdict()`
   - Add to ensemble voting (equal weight to FormalLogicChecker)

3. **Calibrate thresholds** via benchmark
   - Test on HaluEval + TruthfulQA
   - Fine-tune cosine_sim threshold (0.55-0.70 range)
   - Fine-tune elaboration ratio if needed

4. **Add tests** (3-5 test cases)
   - Known elaboration patterns
   - Known semantic drift examples
   - Edge cases (very short knowledge, very long answer)

### Code Changes Summary

**Files to modify**:
- `crates/pure-reason-core/src/ensemble_verifier.rs` (+80 LOC)
- `crates/pure-reason-core/src/pipeline.rs` (+5 LOC, just call)
- `benchmarks/run_downloaded_benchmarks.py` (no change needed, rerun after)

**Expected final state**:
- 276 + 80 = 356 LOC in ensemble_verifier.rs (still <400 ✓)
- 5 detectors → 6 detectors
- 269 tests → 272-275 tests (3-5 new tests)

---

## Calibration Strategy

### Threshold Tuning (per benchmark)

We'll use a simple heuristic: **what threshold gives best F1 on benchmark dev set?**

```python
# Pseudo-code for calibration
for threshold in [0.50, 0.55, 0.60, 0.65, 0.70, 0.75]:
    predictions = ensemble_with_semantic_sim(threshold)
    f1 = compute_f1(predictions, ground_truth)
    print(f"Threshold {threshold}: F1 = {f1:.3f}")

# Pick threshold with highest F1
```

**Benchmarks to use**:
- TruthfulQA (strong on semantic consistency)
- HaluEval (elaboration patterns)
- HalluLens (already +0.113, validate no regression)

### Expected Results After A2

| Benchmark | Before A2 | After A2 | Delta |
|-----------|-----------|----------|-------|
| HalluLens | 0.762 | 0.780 | +0.018 |
| LogicBench | 0.821 | 0.835 | +0.014 |
| TruthfulQA | 0.812 | 0.830 | +0.018 |
| **Average** | **0.603** | **0.625** | **+0.022** |

---

## Integration Points

### Where does semantic detector fit?

Currently in `compose_verdict()`:

```rust
pub fn compose_verdict(
    segments: &SegmentedInput,
    // ... other params
) -> Verdict {
    // ... cognitive layers ...
    
    // Phase A: Existing ensemble
    let ensemble_verdict = EnsembleVerifier::verify(&segments.knowledge, &segments.answer);
    
    // Phase A2: Add semantic similarity (NEW)
    let semantic_vote = SemanticSimilarityDetector::check_drift(&segments.knowledge, &segments.answer);
    
    // Aggregate (weighted voting already handles variable detector counts)
    let final_confidence = ensemble_verdict.hallucination_probability; // already includes semantic
    
    Verdict {
        ensemble_confidence: final_confidence,
        // ... other fields ...
    }
}
```

The beauty of the DetectorVote pattern: we just **add another detector**, no refactoring needed.

---

## Risk Assessment

### What could go wrong?

1. **spaCy vectors not available** in deployment
   - Mitigation: Graceful fallback (confidence = 0.3, flags_risk = false)

2. **Similarity threshold too strict** (false positives)
   - Mitigation: Conservative default (0.65), tune down if needed
   - Won't hurt Phase A; worst case excludes this detector

3. **spaCy word vectors poor on domain vocab** (science, medicine)
   - Mitigation: Complement with NumericDomainDetector + FormalLogicChecker

4. **Performance impact** (vectors are expensive to compute)
   - Mitigation: Vectorize once per text, reuse (already done for knowledge)
   - Expected: <10ms per call (acceptable)

---

## Success Criteria

- [x] Semantic detector implemented (80 LOC, <100 target)
- [ ] Integrated into ensemble voting
- [ ] Tests passing (all 269+ + new tests)
- [ ] Benchmarks validated
  - [ ] No regression on HalluLens (should stay >0.75)
  - [ ] No regression on LogicBench (should stay >0.80)
  - [ ] +0.015-0.025 F1 on TruthfulQA
  - [ ] Average F1: 0.603 → 0.620+
- [ ] Committed with documentation

---

## Next After A2

Once A2 is done and validated:

**Option 1: Phase B (Distilled Model)** — 3-4 weeks
- Train DistilBERT binary classifier ("falsifiable?")
- Ensemble weight: 30% model + 70% heuristics
- ROI: +0.08-0.12 F1 (0.625 → 0.70+)

**Option 2: Phase A3 (Domain Validator)** — 1-2 weeks (if FELM weak)
- Connect NumericDomainDetector to fact DB
- Validate scientific/medical claims
- ROI: +0.03-0.05 F1 on FELM specifically

**Option 3: Phase C (Self-Auditing)** — 2-3 weeks (if 0.70+ achieved)
- Contradiction detection
- Counterfactual reasoning
- Risk surface governance

---

## Files Reference

- **Phase A**: `docs/SCALE2-PHASE-A.md`
- **Main detector**: `crates/pure-reason-core/src/ensemble_verifier.rs` (current 276 LOC)
- **Pipeline**: `crates/pure-reason-core/src/pipeline.rs`
- **Benchmarks**: `benchmarks/run_downloaded_benchmarks.py`
