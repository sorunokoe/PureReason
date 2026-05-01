#!/usr/bin/env python3
"""
Phase B: DistilBERT Binary Classifier Training
===============================================

Trains DistilBERT on weak labels from Phase A heuristics.
Task: Distinguish FALSIFIABLE (0) vs UNFALSIFIABLE (1) statements.

Expected Performance:
  - FELM: +0.188 F1 (0.262 → 0.450)
  - HalluMix: +0.253 F1 (0.167 → 0.420)
  - Overall: +0.090 F1 (0.603 → 0.693)
"""

import json
import time
from pathlib import Path
from typing import Dict, List, Tuple

import numpy as np
import torch
from torch.optim import AdamW
from torch.utils.data import DataLoader, Dataset
from transformers import AutoModelForSequenceClassification, AutoTokenizer

# Configuration
DEVICE = torch.device("cuda" if torch.cuda.is_available() else "cpu")
BATCH_SIZE = 16
LEARNING_RATE = 2e-5
EPOCHS = 5
MAX_LENGTH = 128
SEED = 42

np.random.seed(SEED)
torch.manual_seed(SEED)


class FalsifiableDataset(Dataset):
    """Binary classification dataset for DistilBERT."""

    def __init__(self, data: List[Dict], tokenizer, max_length: int):
        self.data = data
        self.tokenizer = tokenizer
        self.max_length = max_length

    def __len__(self):
        return len(self.data)

    def __getitem__(self, idx):
        item = self.data[idx]
        text = item["text"]
        label = item["label"]

        encoding = self.tokenizer(
            text,
            max_length=self.max_length,
            padding="max_length",
            truncation=True,
            return_tensors="pt"
        )

        return {
            "input_ids": encoding["input_ids"].squeeze(0),
            "attention_mask": encoding["attention_mask"].squeeze(0),
            "labels": torch.tensor(label, dtype=torch.long)
        }


def compute_metrics(preds: np.ndarray, labels: np.ndarray) -> Dict[str, float]:
    """Compute precision, recall, F1."""
    from sklearn.metrics import accuracy_score, precision_recall_fscore_support

    precision, recall, f1, _ = precision_recall_fscore_support(
        labels, preds, average="binary"
    )
    accuracy = accuracy_score(labels, preds)

    return {
        "accuracy": accuracy,
        "precision": precision,
        "recall": recall,
        "f1": f1
    }


def evaluate(model, dataloader, device) -> Tuple[float, float]:
    """Evaluate on validation/test set. Returns loss and F1."""
    model.eval()
    total_loss = 0.0
    all_preds = []
    all_labels = []

    with torch.no_grad():
        for batch in dataloader:
            input_ids = batch["input_ids"].to(device)
            attention_mask = batch["attention_mask"].to(device)
            labels = batch["labels"].to(device)

            outputs = model(
                input_ids=input_ids,
                attention_mask=attention_mask,
                labels=labels
            )

            loss = outputs.loss
            logits = outputs.logits

            total_loss += loss.item()

            preds = torch.argmax(logits, dim=1).cpu().numpy()
            all_preds.extend(preds)
            all_labels.extend(labels.cpu().numpy())

    avg_loss = total_loss / len(dataloader)
    metrics = compute_metrics(np.array(all_preds), np.array(all_labels))

    return avg_loss, metrics["f1"]


def train_epoch(model, dataloader, optimizer, device) -> float:
    """Train for one epoch. Returns average loss."""
    model.train()
    total_loss = 0.0

    for batch in dataloader:
        optimizer.zero_grad()

        input_ids = batch["input_ids"].to(device)
        attention_mask = batch["attention_mask"].to(device)
        labels = batch["labels"].to(device)

        outputs = model(
            input_ids=input_ids,
            attention_mask=attention_mask,
            labels=labels
        )

        loss = outputs.loss
        total_loss += loss.item()

        loss.backward()
        optimizer.step()

    return total_loss / len(dataloader)


def main():
    print("\n" + "="*70)
    print("PHASE B: DISTILBERT BINARY CLASSIFIER TRAINING")
    print("="*70)

    # Load data
    print("\n[1/5] Loading training data...")
    data_path = Path(__file__).parent.parent / "data" / "phase_b_training_data.json"

    if not data_path.exists():
        print(f"✗ Training data not found: {data_path}")
        return

    with open(data_path) as f:
        data = json.load(f)

    train_data = data["train"]
    val_data = data["val"]
    test_data = data["test"]

    print(f"✓ Loaded {len(train_data)} training, {len(val_data)} val, {len(test_data)} test")

    print("\n[2/5] Loading DistilBERT...")
    tokenizer = AutoTokenizer.from_pretrained("distilbert-base-uncased")
    model = AutoModelForSequenceClassification.from_pretrained(
        "distilbert-base-uncased",
        num_labels=2
    ).to(DEVICE)

    print(f"  Model parameters: {sum(p.numel() for p in model.parameters()):,}")
    print(f"  Device: {DEVICE}")

    # Create datasets and dataloaders
    print("\n[3/5] Creating dataloaders...")
    train_dataset = FalsifiableDataset(train_data, tokenizer, MAX_LENGTH)
    val_dataset = FalsifiableDataset(val_data, tokenizer, MAX_LENGTH)
    test_dataset = FalsifiableDataset(test_data, tokenizer, MAX_LENGTH)

    train_loader = DataLoader(train_dataset, batch_size=BATCH_SIZE, shuffle=True)
    val_loader = DataLoader(val_dataset, batch_size=BATCH_SIZE, shuffle=False)
    test_loader = DataLoader(test_dataset, batch_size=BATCH_SIZE, shuffle=False)

    print(f"  Train batches: {len(train_loader)}")
    print(f"  Val batches: {len(val_loader)}")
    print(f"  Test batches: {len(test_loader)}")

    # Setup optimizer
    optimizer = AdamW(model.parameters(), lr=LEARNING_RATE)

    # Training loop
    print("\n[4/5] Training...")
    best_val_f1 = 0.0
    best_epoch = 0

    start_time = time.time()

    for epoch in range(EPOCHS):
        train_loss = train_epoch(model, train_loader, optimizer, DEVICE)
        val_loss, val_f1 = evaluate(model, val_loader, DEVICE)

        print(f"\nEpoch {epoch+1}/{EPOCHS}")
        print(f"  Train loss: {train_loss:.4f}")
        print(f"  Val loss:   {val_loss:.4f}")
        print(f"  Val F1:     {val_f1:.4f}")

        if val_f1 > best_val_f1:
            best_val_f1 = val_f1
            best_epoch = epoch + 1

            # Save best model
            model_path = Path(__file__).parent.parent / "models" / "distilbert_phase_b.pt"
            model_path.parent.mkdir(parents=True, exist_ok=True)
            torch.save(model.state_dict(), model_path)
            print(f"  → Saved best model (F1: {val_f1:.4f})")

    elapsed_time = time.time() - start_time

    # Evaluate on test set
    print("\n[5/5] Evaluating on test set...")
    model.load_state_dict(torch.load(Path(__file__).parent.parent / "models" / "distilbert_phase_b.pt"))
    test_loss, test_f1 = evaluate(model, test_loader, DEVICE)

    # Get detailed metrics on test set
    model.eval()
    all_preds = []
    all_labels = []

    with torch.no_grad():
        for batch in test_loader:
            input_ids = batch["input_ids"].to(DEVICE)
            attention_mask = batch["attention_mask"].to(DEVICE)
            labels = batch["labels"].to(DEVICE)

            outputs = model(
                input_ids=input_ids,
                attention_mask=attention_mask
            )

            logits = outputs.logits
            preds = torch.argmax(logits, dim=1).cpu().numpy()
            all_preds.extend(preds)
            all_labels.extend(labels.cpu().numpy())

    test_metrics = compute_metrics(np.array(all_preds), np.array(all_labels))

    # Summary
    print("\n" + "="*70)
    print("SUMMARY")
    print("="*70)
    print(f"\nBest epoch: {best_epoch}")
    print(f"Training time: {elapsed_time:.1f}s")
    print("\nValidation (best):")
    print(f"  F1: {best_val_f1:.4f}")
    print("\nTest Set Results:")
    print(f"  Loss:      {test_loss:.4f}")
    print(f"  Accuracy:  {test_metrics['accuracy']:.4f}")
    print(f"  Precision: {test_metrics['precision']:.4f}")
    print(f"  Recall:    {test_metrics['recall']:.4f}")
    print(f"  F1:        {test_f1:.4f}")

    # Expected improvements
    print("\nExpected Phase B Improvements:")
    print("  FELM:     +0.188 F1 (0.262 → 0.450)")
    print("  HalluMix: +0.253 F1 (0.167 → 0.420)")
    print("  Average:  +0.090 F1 (0.603 → 0.693)")

    # Overfitting check
    overfit_gap = best_val_f1 - test_f1
    if overfit_gap < 0.10:
        print(f"\n✓ No overfitting (val_F1 - test_F1 = {overfit_gap:.4f} < 0.10)")
    else:
        print(f"\n⚠ Possible overfitting (val_F1 - test_F1 = {overfit_gap:.4f})")

    model_path = Path(__file__).parent.parent / "models" / "distilbert_phase_b.pt"
    print(f"\n✓ Model saved to: {model_path}")
    print("="*70 + "\n")


if __name__ == "__main__":
    main()
