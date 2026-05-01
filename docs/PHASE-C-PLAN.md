# Phase C: Self-Auditing Layer (Planning)

**Status**: PLANNING  
**Trigger**: Phase B complete, 0.677 F1 achieved  
**Target**: +0.050 F1 (0.677 → 0.727 F1)  
**Timeline**: 2-3 weeks  
**Architecture**: Contradiction detection + formal reasoning + governance

---

## Problem Statement

### Current State (After Phase B)
- **Average F1**: 0.677 (67.7%)
- **Remaining gap**: 0.323 to reach 100%
- **Major weaknesses**: 
  - Logic consistency not enforced across claims
  - No counterfactual reasoning (dependency analysis)
  - Domain-specific thresholds not calibrated

### Phase C Goal
Add self-auditing capabilities to catch logical contradictions and complex reasoning failures that Phase A+B heuristics + ML miss.

**Example failures Phase A/B misses:**
```
Knowledge: "All mammals breathe with lungs"
LLM says: "Whales are mammals"
LLM says: "Whales breathe with gills"
← CONTRADICTION (both heuristics and model miss this pattern)
```

---

## Architecture Overview

### Layer 1: Contradiction Detection
**Purpose**: Find pairwise inconsistencies in all claims

**Algorithm**:
1. Extract all claims from answer
2. For each pair: semantic similarity check
3. Flag if one claims X and other claims NOT X
4. Use formal logic rules (negation, quantifiers)

**Input**: Segmented claims (from existing analyzer)  
**Output**: Contradiction pairs with confidence

### Layer 2: Counterfactual Reasoning
**Purpose**: Trace dependencies between claims

**Algorithm**:
1. Parse each claim as: Subject → Predicate → Object
2. Build dependency graph (what entities appear in multiple claims?)
3. For high-confidence contradictions, trace causality
4. Example: "If medicine Y fails, does disease Z explanation still work?"

**Input**: Dependency graph of claims  
**Output**: Risk surface (which claims undermine which)

### Layer 3: Governance + Risk Surface
**Purpose**: Apply domain-specific thresholds and generate audit trails

**Algorithm**:
1. Map each claim to domain (medical, legal, finance, etc.)
2. Apply domain-specific contradiction threshold
   - Medical: strict (0.80+ confidence required)
   - History: permissive (0.50+ OK, disagreement expected)
3. Generate audit trail: "Contradiction detected: claim X conflicts with claim Y via [reasoning]"
4. Final verdict: aggregate contradiction scores

**Input**: Contradiction pairs + domains  
**Output**: Audit log + final risk score

---

## Expected Improvements

### Per-Benchmark Analysis

| Benchmark | Phase B | +Phase C | Delta | Reason |
|-----------|---------|---------|-------|--------|
| **FELM** | 0.626 | 0.680 | +0.054 | Catches self-contradictory arithmetic |
| **HaluEval Dialogue** | 0.602 | 0.660 | +0.058 | Multi-turn contradictions |
| **TruthfulQA** | 0.812 | 0.835 | +0.023 | Some answer pairs contradict knowledge |
| **HalluMix** | 0.667 | 0.715 | +0.048 | Complex contradictions between facts |
| **LogicBench** | 0.821 | 0.845 | +0.024 | Formal logic violations caught |
| **Others** | — | — | — | Stable (no contradiction patterns) |
| **AVERAGE** | **0.677** | **0.727** | **+0.050** | **GOAL** |

### Why These Specific Gains?

1. **FELM (+54)**: Numeric claims often self-contradict (e.g., "X is 5" then "X is 7")
2. **Dialogue (+58)**: Multi-turn sequences can introduce contradictions across turns
3. **TruthfulQA (+23)**: Q+A pairs sometimes contradict the knowledge base
4. **HalluMix (+48)**: Strong on catching compound contradictions

---

## Implementation Plan

### Phase C-1: Contradiction Detection (Week 1)

**Goal**: Extract all claims and find pairwise contradictions

**Files to create:**
- `crates/pure-reason-core/src/contradiction_detector.rs` (~150 LOC)
  - `extract_claims()` — Parse answer into atomic propositions
  - `find_contradictions()` — Pairwise semantic consistency check
  - `ContradictionPair` struct with confidence scores

- `crates/pure-reason-core/src/contradiction_detector/formal_logic.rs` (~80 LOC)
  - `NegationRule` — Handle "NOT X" patterns
  - `QuantifierRule` — Handle "all/some/none" patterns
  - `CausalityRule` — Handle "if-then" violations

**Tests**: 5-8 test cases
- Extract simple claims ("X is Y")
- Extract negations ("X is NOT Y")
- Detect contradictions (X vs NOT X)
- Handle quantifiers (all mammals → whales are mammals)

**Milestone**: extraction + basic contradiction detection working

### Phase C-2: Counterfactual Reasoning (Week 1-2)

**Goal**: Trace claim dependencies and test robustness

**Files to create:**
- `crates/pure-reason-core/src/counterfactual_reasoner.rs` (~120 LOC)
  - `DependencyGraph` — Store subject/predicate/object triples
  - `trace_dependency()` — Follow entity mentions across claims
  - `counterfactual_test()` — "If claim A is false, does claim B still hold?"

**Algorithm**:
```
1. For each claim pair (A, B):
   a. Extract entities from both
   b. If shared entity: check if they depend on each other
   c. Test: "If A is false, is B still consistent?"
   d. High contradiction if: A false → B nonsensical
```

**Tests**: 5-8 test cases
- Simple dependency (medicine X → treat disease Y)
- Multi-hop (drug → side effect → symptom)
- Negated dependency (if NOT X, then?)

**Milestone**: Counterfactual tests working on simple chains

### Phase C-3: Domain Routing + Governance (Week 2)

**Goal**: Apply domain-specific thresholds and generate audit trails

**Files to create:**
- `crates/pure-reason-core/src/domain_governance.rs` (~100 LOC)
  - `DomainThreshold` enum (Medical, Legal, Finance, History, Science)
  - `apply_domain_threshold()` — Adjust contradiction confidence by domain
  - `AuditTrail` — Store reasoning decisions

**Domain Thresholds**:
- **Medical**: 0.80+ (high bar, health-critical)
- **Legal**: 0.75+ (high bar, compliance-critical)
- **Finance**: 0.70+ (medium bar, accuracy important)
- **History**: 0.50+ (low bar, disagreement expected)
- **Science**: 0.60+ (medium bar, reproducibility important)

**Tests**: 3-5 test cases
- Medical contradiction vs legal contradiction (different thresholds)
- Audit trail generation
- Risk aggregation

**Milestone**: Domain-aware scoring working

### Phase C-4: Integration + Testing (Week 2-3)

**Goal**: Integrate into pipeline and validate benchmarks

**Changes**:
- Modify `pipeline.rs` → `compose_verdict()`
  - Add contradiction detection pass
  - Blend Phase B score (70%) + contradiction signal (30%)
  - Update `Verdict` struct with `contradictions: Vec<ContradictionPair>`

- Create integration test: verify contradictions reduce F1 on contradictory datasets

**Benchmarks**:
- Run full 9-benchmark suite
- Per-benchmark analysis of contradiction signals
- Measure improvement vs Phase B

**Milestone**: All 9 benchmarks pass, +0.040-0.060 F1 observed

---

## Success Criteria

### Functional
- [ ] `contradiction_detector.rs` compiles, all tests pass
- [ ] Extracts claims accurately (8/10 test cases ≥ 90% precision)
- [ ] Finds contradictions in known-bad inputs (5/5 test cases)
- [ ] Counterfactual reasoning works on chains (5/5 test cases)
- [ ] Domain thresholds applied correctly (3/5 test cases)
- [ ] Audit trail generated and human-readable

### Performance
- [ ] Contradiction detection: <50 ms per 100-token input
- [ ] Total pipeline latency: <250 ms (was ~200 ms, new +50 ms acceptable)
- [ ] Memory: <100 MB additional

### Quality
- [ ] All 274 existing tests still pass
- [ ] Zero regressions on Phase A/B benchmarks
- [ ] Benchmark improvement: +0.040-0.060 F1 (target: +0.050)
- [ ] At least 4/5 major benchmarks show improvement

---

## Risk Assessment

### Low Risk
- ✓ Builds on existing claims extraction (already proven)
- ✓ Pure Rust implementation (no new dependencies)
- ✓ Can fall back to Phase B if contradiction detection unreliable

### Medium Risk
- ⚠ Formal logic rules may need calibration per domain
- ⚠ Counterfactual reasoning is complex (may underperform on nuanced claims)

### Mitigation
- Start with strict mode (only obvious contradictions)
- Use domain thresholds to avoid false positives in History/Philosophy
- Fallback: if contradiction confidence < 0.60, ignore and use Phase B score only

---

## Not Included (Out of Scope)

- External knowledge base queries (no API calls)
- Fine-grained semantic similarity (would add complexity)
- Machine learning for contradiction detection (keep deterministic)
- Real-time monitoring/logging (add in Phase D)

---

## Timeline

- **Week 1**: Contradiction detection + counterfactual reasoning (Phases C-1 and C-2)
- **Week 2**: Domain governance + integration (Phases C-3 and C-4)
- **Week 3**: Testing, benchmarking, documentation

**Parallel**: Update ADR-002 with Phase C decisions, document failure modes

---

## How This Fits the Grand Vision

**Scale 2 Progress**:
- Phase A: +0.010 F1 (ensemble heuristics)
- Phase A2: +0.022 F1 (semantic similarity) [SKIPPED, subsumed by Phase B]
- Phase B: +0.126 F1 (DistilBERT model) [✅ COMPLETE]
- Phase C: +0.050 F1 (contradiction detection) [THIS PHASE]
- Phase D: +0.030 F1 (production + governance) [FUTURE]

**Cumulative**: 0.551 → 0.677 → 0.727 → 0.757 F1 (target: 0.70-0.85)

---

## Next Decision Point

**If Phase C succeeds** (0.70+ F1 achieved):
- Proceed to Phase D (production integration + monitoring)
- Begin domain-specific fine-tuning

**If Phase C struggles** (< 0.69 F1 achieved):
- Consider alternative: domain-specific models (legal, medical, financial)
- Or: improve Phase B model via more training data
- Keep Phase C as optional validation layer (not primary signal)

