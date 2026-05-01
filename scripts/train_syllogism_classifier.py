#!/usr/bin/env python3
"""Train and save syllogism validity classifier."""

import sys
sys.path.insert(0, '.')

import numpy as np
import pickle
from benchmarks.run_reasoning_verification import _VALID_SYLLOGISMS, _INVALID_SYLLOGISMS
from pureason.reasoning._syllogism_clf import _train_syllogism_classifier

# Prepare training data
valid = list(_VALID_SYLLOGISMS)
invalid = list(_INVALID_SYLLOGISMS)

premises_list = [list(p) for p, _ in valid] + [list(p) for p, _ in invalid]
conclusions = [c for _, c in valid] + [c for _, c in invalid]
labels = [1] * len(valid) + [0] * len(invalid)

print(f"Training syllogism classifier on {len(premises_list)} examples")
print(f"  Valid: {len(valid)}, Invalid: {len(invalid)}")

vectorizer, clf = _train_syllogism_classifier(premises_list, conclusions, labels)

# Save
out_path = 'data/syllogism_clf.pkl'
with open(out_path, 'wb') as f:
    pickle.dump((vectorizer, clf), f)
print(f"Saved to {out_path}")

# Quick validation
from sklearn.metrics import accuracy_score, f1_score
X_test = vectorizer.transform(["\n".join(p) + " | " + c for p, c in zip(premises_list, conclusions)])
preds = clf.predict(X_test)
acc = accuracy_score(labels, preds)
f1 = f1_score(labels, preds)
print(f"Training accuracy: {acc:.3f}, F1: {f1:.3f}")
