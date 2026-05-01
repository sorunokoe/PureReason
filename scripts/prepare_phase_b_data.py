#!/usr/bin/env python3
"""
Phase B Data Preparation
========================

Loads FELM, TruthfulQA, and HaluEval datasets and prepares training data.

Output: training_data.json with structure:
{
    "train": [{"text": "...", "label": 0/1}, ...],
    "val": [...],
    "test": [...]
}
"""

import csv
import json
import random
from pathlib import Path
from typing import Dict, List

# Random seed for reproducibility
random.seed(42)

def load_felm(felm_path: Path) -> List[Dict]:
    """Load FELM dataset (factual and non-factual claims)."""
    samples = []
    try:
        with open(felm_path) as f:
            for line in f:
                if line.strip():
                    entry = json.loads(line)
                    # labels is a list, take first element
                    is_factual = entry.get("labels", [False])[0]
                    samples.append({
                        "claim": entry.get("response", ""),
                        "label": 1 if is_factual else 0,
                        "source": "felm"
                    })
        print(f"✓ Loaded {len(samples)} FELM samples")
        # Show label distribution
        pos = sum(1 for s in samples if s["label"] == 1)
        neg = len(samples) - pos
        print(f"  Distribution: {pos} factual, {neg} non-factual")
    except Exception as e:
        print(f"✗ Failed to load FELM: {e}")
    return samples

def load_truthfulqa(qa_path: Path) -> List[Dict]:
    """Load TruthfulQA CSV and create synthetic pairs."""
    samples = []
    try:
        with open(qa_path) as f:
            reader = csv.DictReader(f)
            for row in reader:
                question = row.get("Question", "")

                # Parse correct and incorrect answers
                correct = row.get("Correct Answers", "")
                incorrect = row.get("Incorrect Answers", "")

                # Parse semicolon-separated lists
                for answer in correct.split(";"):
                    answer = answer.strip()
                    if answer:
                        samples.append({
                            "text": f"{question} {answer}",
                            "label": 1,  # UNFALSIFIABLE
                            "source": "truthfulqa"
                        })

                for answer in incorrect.split(";"):
                    answer = answer.strip()
                    if answer:
                        samples.append({
                            "text": f"{question} {answer}",
                            "label": 0,  # FALSIFIABLE
                            "source": "truthfulqa"
                        })

        print(f"✓ Loaded {len(samples)} TruthfulQA pairs")
        pos = sum(1 for s in samples if s["label"] == 1)
        neg = len(samples) - pos
        print(f"  Distribution: {pos} correct, {neg} incorrect")
    except Exception as e:
        print(f"✗ Failed to load TruthfulQA: {e}")
    return samples

def load_halueval(halueval_path: Path) -> List[Dict]:
    """Load HaluEval JSONL dataset."""
    samples = []
    try:
        with open(halueval_path) as f:
            for line in f:
                if line.strip():
                    entry = json.loads(line)

                    knowledge = entry.get("knowledge", "")
                    question = entry.get("question", "")
                    right_answer = entry.get("right_answer", "")
                    hallucinated_answer = entry.get("hallucinated_answer", "")

                    # Create two samples: one correct, one hallucinated
                    if right_answer:
                        text = f"{knowledge} {question} {right_answer}".strip()
                        samples.append({
                            "text": text,
                            "label": 1,  # Correct answer - UNFALSIFIABLE
                            "source": "halueval"
                        })

                    if hallucinated_answer:
                        text = f"{knowledge} {question} {hallucinated_answer}".strip()
                        samples.append({
                            "text": text,
                            "label": 0,  # Hallucinated answer - FALSIFIABLE
                            "source": "halueval"
                        })

        print(f"✓ Loaded {len(samples)} HaluEval samples")
        pos = sum(1 for s in samples if s["label"] == 1)
        neg = len(samples) - pos
        print(f"  Distribution: {pos} correct, {neg} hallucinated")
    except Exception as e:
        print(f"✗ Failed to load HaluEval: {e}")
    return samples

def format_for_distilbert(text: str) -> str:
    """Format text for DistilBERT input."""
    # Simple format: [CLS] text [SEP]
    return f"[CLS] {text} [SEP]"

def balance_dataset(samples: List[Dict]) -> List[Dict]:
    """Balance dataset by downsampling majority class."""
    positives = [s for s in samples if s["label"] == 1]
    negatives = [s for s in samples if s["label"] == 0]

    print("\n  Original distribution:")
    print(f"    Positive (unfalsifiable): {len(positives)}")
    print(f"    Negative (falsifiable): {len(negatives)}")

    # Downsample majority class to match minority
    min_count = min(len(positives), len(negatives))

    if len(positives) > min_count:
        positives = random.sample(positives, min_count)
    if len(negatives) > min_count:
        negatives = random.sample(negatives, min_count)

    balanced = positives + negatives
    random.shuffle(balanced)

    print("\n  Balanced distribution:")
    print(f"    Positive: {len(positives)}")
    print(f"    Negative: {len(negatives)}")
    print(f"    Total: {len(balanced)}")

    return balanced

def prepare_data(data_dir: Path) -> Dict[str, List[Dict]]:
    """Load all datasets, balance, and split."""

    print("\n" + "="*70)
    print("PHASE B: DATA PREPARATION FOR DISTILBERT")
    print("="*70)

    all_samples = []

    # Load each dataset
    print("\n[1/4] Loading datasets...")

    felm_path = data_dir / "felm" / "all.jsonl"
    if felm_path.exists():
        all_samples.extend(load_felm(felm_path))
    else:
        print(f"✗ FELM not found: {felm_path}")

    qa_path = data_dir / "truthfulqa" / "TruthfulQA.csv"
    if qa_path.exists():
        all_samples.extend(load_truthfulqa(qa_path))
    else:
        print(f"✗ TruthfulQA not found: {qa_path}")

    halueval_path = data_dir / "halueval" / "qa_data.json"
    if halueval_path.exists():
        all_samples.extend(load_halueval(halueval_path))
    else:
        print(f"✗ HaluEval not found: {halueval_path}")

    if not all_samples:
        print("\n✗ No data loaded!")
        return {}

    print(f"\n✓ Total raw samples: {len(all_samples)}")

    # Balance dataset
    print("\n[2/4] Balancing dataset...")
    balanced = balance_dataset(all_samples)

    if not balanced:
        print("\n✗ Balanced dataset is empty!")
        return {}

    # Split into train/val/test (70/10/20)
    print("\n[3/4] Splitting into train/val/test (70/10/20)...")
    total = len(balanced)
    train_size = int(total * 0.70)
    val_size = int(total * 0.10)

    train_data = balanced[:train_size]
    val_data = balanced[train_size:train_size+val_size]
    test_data = balanced[train_size+val_size:]

    print(f"  Train: {len(train_data):5} ({100*len(train_data)/total:5.1f}%)")
    print(f"  Val:   {len(val_data):5} ({100*len(val_data)/total:5.1f}%)")
    print(f"  Test:  {len(test_data):5} ({100*len(test_data)/total:5.1f}%)")

    # Format for DistilBERT
    print("\n[4/4] Formatting for DistilBERT...")

    result = {
        "train": [
            {
                "text": format_for_distilbert(s.get("claim") or s.get("text", "")),
                "label": s["label"],
                "source": s.get("source", "unknown")
            }
            for s in train_data
        ],
        "val": [
            {
                "text": format_for_distilbert(s.get("claim") or s.get("text", "")),
                "label": s["label"],
                "source": s.get("source", "unknown")
            }
            for s in val_data
        ],
        "test": [
            {
                "text": format_for_distilbert(s.get("claim") or s.get("text", "")),
                "label": s["label"],
                "source": s.get("source", "unknown")
            }
            for s in test_data
        ]
    }

    return result

if __name__ == "__main__":
    data_dir = Path(__file__).parent.parent / "benchmarks" / "downloads"

    if not data_dir.exists():
        print(f"Error: Data directory not found: {data_dir}")
        exit(1)

    # Prepare data
    training_data = prepare_data(data_dir)

    if training_data:
        # Save to file
        output_file = Path(__file__).parent.parent / "data" / "phase_b_training_data.json"
        output_file.parent.mkdir(parents=True, exist_ok=True)

        with open(output_file, "w") as f:
            json.dump(training_data, f, indent=2)

        # Print summary
        print("\n" + "="*70)
        print("SUMMARY")
        print("="*70)

        for split in ["train", "val", "test"]:
            data = training_data[split]
            pos = sum(1 for d in data if d["label"] == 1)
            neg = len(data) - pos
            print(f"\n{split.upper()}:")
            print(f"  Total: {len(data)}")
            print(f"  Positive (unfalsifiable): {pos:4} ({100*pos/len(data):5.1f}%)")
            print(f"  Negative (falsifiable):   {neg:4} ({100*neg/len(data):5.1f}%)")

        print("\n" + "="*70)
        print(f"✓ Training data saved to: {output_file}")
        print(f"  File size: {output_file.stat().st_size / 1024 / 1024:.2f} MB")
        print("="*70)
    else:
        print("\n✗ Failed to prepare training data")
        exit(1)
