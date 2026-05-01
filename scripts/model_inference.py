#!/usr/bin/env python3
"""
Phase B: DistilBERT Model Inference Service
============================================

Provides inference interface for the trained DistilBERT classifier.
Used by pure-reason-core pipeline to get model predictions.
"""

import json
import sys
from pathlib import Path

import torch
from transformers import AutoModelForSequenceClassification, AutoTokenizer

# Global model cache
_tokenizer = None
_model = None
_device = None


def load_model():
    """Load DistilBERT model and tokenizer from saved checkpoint."""
    global _tokenizer, _model, _device

    if _model is not None:
        return  # Already loaded

    _device = torch.device("cpu")

    model_path = Path(__file__).parent.parent / "models" / "distilbert_phase_b.pt"

    if not model_path.exists():
        raise FileNotFoundError(f"Model not found: {model_path}")

    try:
        # Load tokenizer and model
        _tokenizer = AutoTokenizer.from_pretrained("distilbert-base-uncased")
        _model = AutoModelForSequenceClassification.from_pretrained(
            "distilbert-base-uncased",
            num_labels=2
        ).to(_device)

        # Load trained weights
        state_dict = torch.load(model_path, map_location=_device)
        _model.load_state_dict(state_dict)
        _model.eval()

        print(f"✓ Model loaded from {model_path}", file=sys.stderr)
    except Exception as e:
        raise RuntimeError(f"Failed to load model: {e}")


def infer(knowledge: str, claim: str) -> dict[str, float]:
    """
    Infer probability that claim is FALSIFIABLE (can be fact-checked, potentially false).

    Args:
        knowledge: Background knowledge or context
        claim: The claim to evaluate

    Returns:
        {
            "falsifiable_prob": 0.0-1.0,     # P(can be fact-checked)
            "unfalsifiable_prob": 0.0-1.0,   # P(correct/unfalsifiable)
            "confidence": 0.0-1.0             # Model confidence
        }
    """
    load_model()

    # Prepare input
    text = f"[CLS] {knowledge} {claim} [SEP]".strip()

    try:
        encoding = _tokenizer(
            text,
            max_length=128,
            padding="max_length",
            truncation=True,
            return_tensors="pt"
        )

        input_ids = encoding["input_ids"].to(_device)
        attention_mask = encoding["attention_mask"].to(_device)

        # Inference
        with torch.no_grad():
            outputs = _model(
                input_ids=input_ids,
                attention_mask=attention_mask
            )
            logits = outputs.logits
            probs = torch.softmax(logits, dim=1)[0].cpu().numpy()

        # Label mapping
        # 0 = FALSIFIABLE (can be fact-checked)
        # 1 = UNFALSIFIABLE (correct/established)
        falsifiable_prob = float(probs[0])
        unfalsifiable_prob = float(probs[1])
        confidence = max(falsifiable_prob, unfalsifiable_prob)

        return {
            "falsifiable_prob": falsifiable_prob,
            "unfalsifiable_prob": unfalsifiable_prob,
            "confidence": confidence
        }

    except Exception as e:
        raise RuntimeError(f"Inference failed: {e}")


def main():
    """Test inference on command-line input."""
    if len(sys.argv) < 3:
        print("Usage: python3 model_inference.py <knowledge> <claim>", file=sys.stderr)
        sys.exit(1)

    knowledge = sys.argv[1]
    claim = sys.argv[2]

    result = infer(knowledge, claim)
    print(json.dumps(result))


if __name__ == "__main__":
    main()
