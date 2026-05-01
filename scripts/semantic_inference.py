#!/usr/bin/env python3
"""
Semantic Fallback Inference Service
====================================

Provides sentence-transformers inference for semantic similarity detection.
Used by pure-reason-core semantic_fallback.rs module.

Model: all-MiniLM-L6-v2 (22MB, same as FELM benchmark)
"""

import json
import sys

try:
    import numpy as np
    from sentence_transformers import SentenceTransformer

    HAS_SENTENCE_TRANSFORMERS = True
except ImportError:
    HAS_SENTENCE_TRANSFORMERS = False

# Global model cache
_model = None


def load_model():
    """Load sentence-transformers model (lazy, cached)."""
    global _model

    if not HAS_SENTENCE_TRANSFORMERS:
        raise ImportError(
            "sentence-transformers not installed. Install with: pip install sentence-transformers"
        )

    if _model is not None:
        return  # Already loaded

    try:
        _model = SentenceTransformer("all-MiniLM-L6-v2")
        print("✓ Model loaded: all-MiniLM-L6-v2", file=sys.stderr)
    except Exception as e:
        raise RuntimeError(f"Failed to load model: {e}") from e


def compute_similarity(text1: str, text2: str) -> float:
    """
    Compute cosine similarity between two texts.

    Args:
        text1: First text (knowledge/reference)
        text2: Second text (answer/claim)

    Returns:
        Cosine similarity score (0.0-1.0)
    """
    load_model()

    try:
        # Encode both texts
        embeddings = _model.encode([text1, text2])

        # Compute cosine similarity
        vec1 = embeddings[0]
        vec2 = embeddings[1]

        dot_product = np.dot(vec1, vec2)
        norm1 = np.linalg.norm(vec1)
        norm2 = np.linalg.norm(vec2)

        if norm1 == 0 or norm2 == 0:
            return 0.0

        similarity = float(dot_product / (norm1 * norm2))

        # Clamp to [0, 1]
        return max(0.0, min(1.0, similarity))

    except Exception as e:
        raise RuntimeError(f"Similarity computation failed: {e}") from e


def batch_compute_similarity(pairs: list[tuple[str, str]]) -> list[float]:
    """
    Compute cosine similarities for multiple text pairs (more efficient).

    Args:
        pairs: List of (text1, text2) tuples

    Returns:
        List of similarity scores
    """
    load_model()

    try:
        # Flatten pairs
        texts = []
        for t1, t2 in pairs:
            texts.extend([t1, t2])

        # Batch encode
        embeddings = _model.encode(texts)

        # Compute similarities
        similarities = []
        for i in range(0, len(embeddings), 2):
            vec1 = embeddings[i]
            vec2 = embeddings[i + 1]

            dot_product = np.dot(vec1, vec2)
            norm1 = np.linalg.norm(vec1)
            norm2 = np.linalg.norm(vec2)

            if norm1 == 0 or norm2 == 0:
                similarities.append(0.0)
            else:
                sim = float(dot_product / (norm1 * norm2))
                similarities.append(max(0.0, min(1.0, sim)))

        return similarities

    except Exception as e:
        raise RuntimeError(f"Batch similarity computation failed: {e}") from e


def main():
    """Command-line interface for similarity computation."""
    if len(sys.argv) < 3:
        print("Usage: python3 semantic_inference.py <text1> <text2>", file=sys.stderr)
        sys.exit(1)

    text1 = sys.argv[1]
    text2 = sys.argv[2]

    try:
        similarity = compute_similarity(text1, text2)
        result = {"similarity": similarity, "threshold": 0.86, "flags_risk": similarity < 0.86}
        print(json.dumps(result))
    except Exception as e:
        print(json.dumps({"error": str(e)}), file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
