# Meta-Learner V2 Architecture Design

**Date**: 2026-05-01  
**Status**: Design Complete  
**TRIZ Principles**: P13 (Locally Rapid Action), P2 (Taking Out)

---

## Overview

Session-scoped adaptive learner that tracks detector accuracy and adjusts ensemble weights in real-time. **No persistence** - learning resets between sessions to maintain determinism for benchmarks.

---

## Key Features

- **Session-scoped**: Learning confined to single verification session
- **Detector accuracy tracking**: Per-detector (hits, misses) counters
- **Adaptive weights**: Every 100 calls, rebalance ensemble weights
- **Exponential smoothing**: α=0.3 to avoid wild swings
- **Warmup period**: First 100 calls use default weights

---

## Architecture

```rust
pub struct SessionMetaLearner {
    /// Per-detector accuracy: (detector_name → (hits, misses))
    detector_stats: HashMap<String, (usize, usize)>,
    
    /// Current ensemble weights (adaptive)
    weights: EnsembleWeights,
    
    /// Default weights (fallback)
    default_weights: EnsembleWeights,
    
    /// Call counter
    call_count: usize,
    
    /// Warmup period (calls before adaptation begins)
    warmup_calls: usize,
    
    /// Adaptation frequency (recalculate weights every N calls)
    adapt_frequency: usize,
    
    /// Exponential smoothing factor (0.0-1.0)
    alpha: f64,
}
```

---

## Workflow

```
1. Session starts → Initialize SessionMetaLearner
2. First 100 calls → Use default weights, track detector accuracy
3. Call 100 → Compute adaptive weights (first adaptation)
4. Calls 101-200 → Use adaptive weights, continue tracking
5. Call 200 → Recompute adaptive weights (second adaptation)
6. Repeat every 100 calls
7. Session ends → Discard SessionMetaLearner (no persistence)
```

---

## Weight Adaptation Algorithm

### Input
- `detector_stats`: Map of (detector_name → (hits, misses))
- `default_weights`: Starting point

### Output
- `adaptive_weights`: New ensemble weights

### Algorithm

```python
for detector_name in detectors:
    hits, misses = detector_stats[detector_name]
    total = hits + misses
    
    if total == 0:
        # No data yet, use default
        new_weight = default_weights[detector_name]
    else:
        # Compute accuracy
        accuracy = hits / total
        
        # Scale weight by accuracy (1.0 accuracy = 2× default, 0.5 accuracy = 1× default)
        new_weight = default_weights[detector_name] * (1.0 + accuracy)
    
    # Exponential smoothing to avoid wild swings
    adaptive_weights[detector_name] = (
        alpha * new_weight + 
        (1 - alpha) * current_weights[detector_name]
    )
```

**Exponential smoothing** prevents single outliers from dominating. α=0.3 means new weight contributes 30%, old weight 70%.

---

## Integration with VerifierService

```rust
pub struct VerifierService {
    pipeline: KantianPipeline,
    validator: StructuredDecisionValidator,
    trace_store: Option<TraceStore>,
    pre_verifier: PreVerifier,
    
    // New: Meta-learner (session-scoped)
    meta_learner: Arc<Mutex<Option<SessionMetaLearner>>>,
}

impl VerifierService {
    pub fn verify(&self, req: VerificationRequest) -> Result<VerificationResult> {
        // Initialize meta-learner on first call if trace_id present
        if req.trace_id.is_some() {
            let mut ml_guard = self.meta_learner.lock().unwrap();
            if ml_guard.is_none() {
                *ml_guard = Some(SessionMetaLearner::new());
            }
        }
        
        // Get current weights
        let weights = {
            let ml_guard = self.meta_learner.lock().unwrap();
            ml_guard.as_ref().map(|ml| ml.get_weights()).unwrap_or_default()
        };
        
        // Run verification with adaptive weights
        let result = self.verify_with_weights(&req, weights)?;
        
        // Update meta-learner with result
        if let Some(ml) = self.meta_learner.lock().unwrap().as_mut() {
            ml.update_after_verification(&result.detector_votes, result.actual_verdict);
        }
        
        Ok(result)
    }
}
```

---

## Expected Impact

**Target**: +5-10pp F1 improvement after 100-call warmup

**Mechanism**:
- Detectors that consistently perform well get higher weights
- Detectors that consistently fail get lower weights
- Domain-specific patterns emerge (e.g., numeric detector excels in medical domain)

**Example**:
- Initial weights: all detectors 1.0
- After 100 medical claims:
  - Numeric detector: 0.95 accuracy → weight 1.85
  - Semantic detector: 0.60 accuracy → weight 1.20
  - KAC detector: 0.80 accuracy → weight 1.60
- Result: Medical claims verified more accurately (numeric errors caught)

---

## Testing Strategy

### Unit Tests
1. **Weight adaptation correctness**: Mock scenarios with known outcomes
2. **Exponential smoothing behavior**: Verify smoothing prevents wild swings
3. **Warmup period**: First 100 calls use default weights
4. **Session isolation**: Multiple sessions don't contaminate each other

### Integration Tests
1. **Session lifecycle**: Initialize → adapt → reset
2. **Concurrent sessions**: Verify thread safety
3. **Benchmark improvement**: F1 gain after warmup (+5pp minimum)

---

## Configuration

```rust
pub struct MetaLearnerConfig {
    /// Warmup period (calls before adaptation)
    pub warmup_calls: usize,  // default: 100
    
    /// Adaptation frequency (recalculate every N calls)
    pub adapt_frequency: usize,  // default: 100
    
    /// Exponential smoothing factor (0.0-1.0)
    pub alpha: f64,  // default: 0.3
    
    /// Minimum calls per detector before trusting accuracy
    pub min_samples: usize,  // default: 10
}
```

---

## Limitations & Tradeoffs

### ✅ Advantages
- No persistence = no benchmark leakage
- Fast adaptation (100 calls)
- Deterministic within session
- Thread-safe (Arc<Mutex>)

### ❌ Limitations
- Resets between sessions (doesn't learn across restarts)
- Requires 100-call warmup (slow cold start)
- Only works when trace_id provided (session tracking)
- Exponential smoothing adds latency (~1-2μs per call)

---

## Alternative Designs Considered

### 1. Persistent Meta-Learner
**Pros**: Learn across sessions  
**Cons**: Benchmark leakage risk, non-deterministic  
**Decision**: Rejected for Phase 1 (may revisit in Phase 2 with leak audit)

### 2. Per-Domain Meta-Learner
**Pros**: Domain-specific weights  
**Cons**: Requires domain detection, complex state management  
**Decision**: Defer to domain_calibration.rs (separate concern)

### 3. Bayesian Weight Updater
**Pros**: Theoretically optimal  
**Cons**: Complex, overkill for this use case  
**Decision**: Rejected (exponential smoothing sufficient)

---

## Success Criteria

- ✅ F1 improvement: +5pp after 100-call warmup
- ✅ Latency overhead: <5μs per call
- ✅ Memory footprint: <10KB per session
- ✅ Thread safety: No deadlocks, no race conditions
- ✅ Session isolation: No cross-contamination

---

**END OF DESIGN**
