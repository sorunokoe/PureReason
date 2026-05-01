# Domain Calibration Format Design

**Date**: 2026-05-01  
**Status**: Design Complete  
**TRIZ Principles**: P3 (Local Quality), P4 (Asymmetry)

---

## Overview

Per-domain YAML configuration files that customize ensemble weights, thresholds, and ECS calibration curves for specific domains (medical, legal, financial, general).

---

## Directory Structure

```
domains/
├── medical.yaml          # Medical/healthcare domain
├── legal.yaml            # Legal/regulatory domain
├── financial.yaml        # Financial/investment domain
├── general.yaml          # General-purpose (default)
├── scientific.yaml       # Scientific research
└── technical.yaml        # Technical documentation
```

---

## YAML Schema

```yaml
# domains/medical.yaml
version: "1.0"
domain: "medical"
description: "Medical and healthcare domain configuration"

# Active domain detection patterns (regex)
detection:
  patterns:
    - "\\b(patient|diagnosis|treatment|prognosis|symptoms?|disease)\\b"
    - "\\b(medical|clinical|pharmaceutical|therapeutic)\\b"
  confidence_threshold: 0.7  # Min confidence to apply this domain config

# Ensemble detector weights
ensemble_weights:
  kac_detector: 1.5          # Knowledge-Answer Contradiction (higher for medical)
  numeric_detector: 2.0      # Numeric plausibility (critical for dosing)
  semantic_detector: 1.2     # Semantic drift
  novelty_detector: 1.3      # Entity novelty
  contradiction_detector: 1.8  # Internal contradictions (dangerous in medical)

# Risk thresholds (domain-specific)
risk_thresholds:
  critical: 0.85    # Above this = block (medical is conservative)
  high: 0.70        # Above this = escalate
  medium: 0.40      # Above this = warn
  low: 0.20         # Below this = pass

# ECS calibration curve (Platt scaling parameters)
calibration:
  method: "platt_scaling"  # or "isotonic_regression"
  # Logistic regression: calibrated_score = 1 / (1 + exp(-(A * raw_score + B)))
  parameters:
    A: 1.35   # Slope (fitted on medical benchmark)
    B: -0.42  # Intercept (fitted on medical benchmark)
  # Calibration data source
  fitted_on:
    benchmark: "medical_claims_200"
    date: "2026-05-01"
    n_samples: 200
    holdout_accuracy: 0.93  # ±5pp ECS accuracy on holdout set

# Confidence bands (domain-specific interpretation)
confidence_bands:
  HIGH:       [80, 100]   # Medical requires high confidence to be "HIGH"
  MODERATE:   [60, 79]
  LOW:        [40, 59]
  VERY_LOW:   [0, 39]

# Domain-specific overrides
overrides:
  # Disable world priors for medical (too general)
  disable_world_priors: true
  # Require evidence for claims
  require_evidence: true
  # Strict numeric validation
  strict_numeric_validation: true
```

---

## Calibration Curve Fitting Process

### 1. Collect Calibration Data

```bash
python3 benchmarks/collect_calibration_data.py \
  --domain medical \
  --benchmark medical_claims \
  --n-samples 400 \
  --output domains/medical_calibration_data.json
```

Produces:
```json
[
  {"raw_score": 0.85, "actual": true},
  {"raw_score": 0.62, "actual": false},
  ...
]
```

### 2. Fit Platt Scaling Curve

```bash
python3 scripts/fit_calibration_curve.py \
  --input domains/medical_calibration_data.json \
  --method platt_scaling \
  --train-samples 200 \
  --holdout-samples 200 \
  --output domains/medical.yaml
```

Fits logistic regression:
```
calibrated_score = 1 / (1 + exp(-(A * raw_score + B)))
```

Where A, B are learned from training data (samples 1-200).

### 3. Validate on Holdout

Holdout samples (201-400) used to measure calibration accuracy:
```
Expected: ECS drift ±5pp (vs ±15pp before calibration)
```

---

## Domain Detection Algorithm

```rust
pub fn detect_domain(text: &str) -> Option<(String, f64)> {
    let text_lower = text.to_lowercase();
    
    for (domain_name, domain_config) in load_all_domains() {
        let mut match_count = 0;
        let total_patterns = domain_config.detection.patterns.len();
        
        for pattern in &domain_config.detection.patterns {
            let regex = Regex::new(pattern)?;
            if regex.is_match(&text_lower) {
                match_count += 1;
            }
        }
        
        let confidence = match_count as f64 / total_patterns as f64;
        
        if confidence >= domain_config.detection.confidence_threshold {
            return Some((domain_name, confidence));
        }
    }
    
    // Fallback to general domain
    Some(("general".to_string(), 1.0))
}
```

---

## Integration with VerifierService

```rust
pub struct VerifierService {
    pipeline: KantianPipeline,
    validator: StructuredDecisionValidator,
    trace_store: Option<TraceStore>,
    pre_verifier: PreVerifier,
    meta_learner: Arc<Mutex<Option<SessionMetaLearner>>>,
    
    // New: Domain calibration
    domain_configs: Arc<HashMap<String, DomainConfig>>,
}

impl VerifierService {
    pub fn verify(&self, req: VerificationRequest) -> Result<VerificationResult> {
        // Detect domain
        let (domain, confidence) = detect_domain(&req.content).unwrap_or(("general".to_string(), 1.0));
        
        // Load domain config
        let domain_config = self.domain_configs.get(&domain).cloned().unwrap_or_default();
        
        // Apply domain-specific weights
        let weights = domain_config.ensemble_weights.clone();
        
        // Run verification with domain weights
        let mut result = self.verify_with_weights(&req, weights)?;
        
        // Apply calibration curve to ECS
        result.verdict.risk_score = domain_config.calibrate_ecs(result.verdict.risk_score);
        
        // Add domain metadata
        result.metadata.insert("domain", json!({
            "detected": domain,
            "confidence": confidence,
            "config_version": domain_config.version,
        }));
        
        Ok(result)
    }
}
```

---

## Example Domain Configs

### Medical (medical.yaml)
- **Conservative**: High thresholds, strict validation
- **Numeric focus**: 2.0× weight on numeric_detector
- **Contradiction-sensitive**: 1.8× weight on contradiction_detector

### Legal (legal.yaml)
- **Citation-focused**: High weight on evidence_detector
- **Precedent-aware**: Custom patterns for case citations
- **Date-sensitive**: Strict temporal coherence

### Financial (financial.yaml)
- **Numeric-centric**: 2.5× weight on numeric_detector
- **Risk-averse**: Higher critical threshold (0.90)
- **Fraud-pattern detection**: Custom blacklist patterns

### General (general.yaml)
- **Balanced**: Equal weights (all 1.0)
- **Moderate thresholds**: Standard risk bands
- **No overrides**: All features enabled

---

## Calibration Data Generation

### Benchmark Setup

For each domain, create a benchmark with 400 labeled claims:
- 200 training samples (fit calibration curve)
- 200 holdout samples (validate calibration)

### Data Collection Script

```python
# benchmarks/collect_calibration_data.py
def collect_calibration_data(domain: str, n_samples: int = 400):
    verifier = VerifierService.new()
    data = []
    
    for claim, actual_verdict in load_benchmark(domain, n_samples):
        result = verifier.verify_text(claim)
        raw_score = result.verdict.risk_score
        
        data.append({
            "claim": claim,
            "raw_score": raw_score,
            "actual": actual_verdict,
        })
    
    return data
```

### Platt Scaling Fitter

```python
# scripts/fit_calibration_curve.py
from sklearn.linear_model import LogisticRegression
import numpy as np

def fit_platt_scaling(train_data):
    X = np.array([d["raw_score"] for d in train_data]).reshape(-1, 1)
    y = np.array([d["actual"] for d in train_data])
    
    model = LogisticRegression()
    model.fit(X, y)
    
    A = model.coef_[0][0]
    B = model.intercept_[0]
    
    return {"A": A, "B": B}
```

---

## Expected Impact

**Target**: ±5pp ECS accuracy across domains (vs ±15pp before)

**Mechanism**:
- Domain-specific calibration curves correct systematic biases
- Medical domain: tends to underestimate risk → calibration increases scores
- General domain: well-calibrated → minimal adjustment

**Example**:
- Raw ECS: 0.75 (medical claim)
- Uncalibrated: Interpreted as "MODERATE" confidence
- Calibrated: 0.82 → "HIGH" confidence (more appropriate for medical)

---

## Success Criteria

- ✅ ECS accuracy: ±5pp drift on holdout set (vs ±15pp before)
- ✅ Domain detection accuracy: >90% precision
- ✅ Latency overhead: <2ms (domain detection + calibration)
- ✅ Config loading: Lazy (only on first use)
- ✅ Backward compatibility: Falls back to general.yaml if domain config missing

---

**END OF DESIGN**
