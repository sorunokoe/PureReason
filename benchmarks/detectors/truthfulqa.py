"""TruthfulQA benchmark.

TRIZ-42 NE-1 remediation
------------------------
The previous revision of this module built a "myth atlas" by loading every
``Incorrect Answers`` field from ``TruthfulQA.csv`` into a trigram database
(``_load_truthfulqa_myths``), then used that database to score predictions on
pairs *sampled from the same CSV*.  Because each test hallucination is
generated from the first entry of its row's ``Incorrect Answers`` column, the
myth DB always contained the ground-truth myth for every test item and the
detector trivially matched it to itself.  That is direct test-set label
leakage.

The verdict function has been reverted to a benchmark-agnostic rule that uses
only the signals produced by ``pure-reason analyze`` on the input text.  No
part of the benchmark CSV is read by the detector any more.

The resulting F1 will be lower than the previously published 0.798; that is
the honest number.  Future work (see TRIZ-42 §10, Bi-system) is to replace
the removed atlas with an *external* misconception corpus (Wikipedia's "List
of common misconceptions" or similar) pinned by content hash — ``world_priors``
is not allowed to be populated from any benchmark's test split.
"""

import csv
import random
from pathlib import Path

from .core import text_value
from .metrics import evaluate_pairs_combined, package_result
from .verdicts import universal_verdict


def benchmark_truthfulqa(downloads_dir: Path, n_per_class: int, seed: int, workers: int) -> dict:
    path = downloads_dir / "truthfulqa" / "TruthfulQA.csv"
    with open(path, newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))

    def make_pair(row: dict) -> list[tuple[str, bool, str]]:
        question = text_value(row.get("Question"))
        best = text_value(row.get("Best Answer"))
        incorrect = text_value(row.get("Incorrect Answers"))
        pairs = []
        if question and best:
            pairs.append((f"Question: {question}\nAnswer: {best}", False, "correct"))
        if question and incorrect:
            wrong = incorrect.split(";")[0].strip()
            if wrong:
                pairs.append((f"Question: {question}\nAnswer: {wrong}", True, "hallucination"))
        return pairs

    flat_pairs = []
    for row in rows:
        flat_pairs.extend(make_pair(row))
    positives = [pair for pair in flat_pairs if pair[1]]
    negatives = [pair for pair in flat_pairs if not pair[1]]
    rng = random.Random(seed)
    rng.shuffle(positives)
    rng.shuffle(negatives)
    n = min(n_per_class, len(positives), len(negatives))
    pairs = positives[:n] + negatives[:n]

    print(f"\n{'=' * 60}\nOFFICIAL BENCHMARK: TruthfulQA\n{'=' * 60}")
    print(f"  Dataset: {path}")
    print(f"  Running on {len(pairs)} samples ({n} correct + {n} hallucinated)...")
    print("  Mode: universal_verdict (post-TRIZ-42: no test-set-derived myth atlas)")

    results = evaluate_pairs_combined(pairs, workers, universal_verdict)
    return package_result(results, "TruthfulQA official", str(path), n)
