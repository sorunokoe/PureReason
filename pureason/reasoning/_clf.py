"""Arithmetic operation classifier — TF-IDF + logistic regression, numpy inference.

Weights are serialized to ``data/op_classifier.npz`` by
``scripts/train_op_classifier.py``.  At inference time this module requires
only ``numpy`` — no ``sklearn`` dependency.
"""

from __future__ import annotations

import math
import re
from pathlib import Path

_CLF_PATH = Path(__file__).parent.parent.parent / "data" / "op_classifier.npz"
_CLF_CACHE: dict | None = None


def _load_clf() -> dict | None:
    """Load serialized classifier weights.

    Returns a dict with ``vocab``, ``idf``, ``coef``, ``intercept``,
    ``classes``, ``stop_words`` or ``None`` if the weights file is absent.
    """
    global _CLF_CACHE
    if _CLF_CACHE is not None:
        return _CLF_CACHE
    if not _CLF_PATH.exists():
        return None
    try:
        import numpy as np

        data = np.load(_CLF_PATH, allow_pickle=True)
        _CLF_CACHE = {
            "vocab": {term: i for i, term in enumerate(data["vocab"])},
            "idf": data["idf"].astype(float),
            "coef": data["coef"].astype(float),
            "intercept": data["intercept"].astype(float),
            "classes": list(data["classes"]),
            "stop_words": set(data["stop_words"]) if "stop_words" in data else set(),
        }
        return _CLF_CACHE
    except Exception:
        return None


def clf_predict(lemma_text: str) -> str | None:
    """Predict the arithmetic operation from spaCy-lemmatized text.

    Implements ``TfidfVectorizer(ngram_range=(1,2), sublinear_tf=True,
    stop_words='english', token_pattern=r'(?u)\\b[a-zA-Z][a-zA-Z]+\\b')``
    + ``LogisticRegression`` in pure numpy.

    Returns one of ``'+'``, ``'-'``, ``'*'``, ``'/'`` or ``None`` if the
    weights file is absent or inference fails.
    """
    clf = _load_clf()
    if clf is None:
        return None
    try:
        import numpy as np

        vocab, idf, coef, intercept, classes, stop_words = (
            clf["vocab"],
            clf["idf"],
            clf["coef"],
            clf["intercept"],
            clf["classes"],
            clf["stop_words"],
        )

        raw_tokens = re.findall(r"(?u)\b[a-zA-Z][a-zA-Z]+\b", lemma_text)
        tokens = [t for t in raw_tokens if t not in stop_words]
        ngrams = list(tokens)
        ngrams += [f"{tokens[i]} {tokens[i + 1]}" for i in range(len(tokens) - 1)]

        counts: dict[int, int] = {}
        for gram in ngrams:
            if gram in vocab:
                idx = vocab[gram]
                counts[idx] = counts.get(idx, 0) + 1

        if not counts:
            return None

        x = np.zeros(len(idf), dtype=float)
        for idx, cnt in counts.items():
            x[idx] = (1.0 + math.log(cnt)) * idf[idx]

        norm = float(np.linalg.norm(x))
        if norm > 0:
            x /= norm

        logits = coef @ x + intercept
        return classes[int(np.argmax(logits))]
    except Exception:
        return None
