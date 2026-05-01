"""LogicBench propositional logic benchmark."""

import json
import random
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from .core import BenchmarkExecutionError, run_pipeline_heuristic, text_value, truncate_text
from .metrics import package_result
from .oracles import (
    logicbench_axiom_oracle,
    logicbench_disjunctive_oracle,
    logicbench_entity_oracle,
    logicbench_pronoun_oracle,
    logicbench_structural_oracle,
    logicbench_vocab_oracle,
)

_LOGICBENCH_FILES = [
    "modus_tollens.json",
    "disjunctive_syllogism.json",
    "hypothetical_syllogism.json",
    "constructive_dilemma.json",
    "destructive_dilemma.json",
    "bidirectional_dilemma.json",
    "material_implication.json",
]


def benchmark_logicbench(downloads_dir: Path, n_per_class: int, seed: int, workers: int) -> dict:
    """Evaluate on LogicBench propositional logic subset (MCQA format).

    Each sample has: context, question, choices (A/B/C/D), answer (correct letter).

    Evaluation protocol:
    - POSITIVE (hallucinated/wrong): context + question + wrong_choice text.
      A wrong conclusion in a propositional logic context should trigger
      antinomy/paralogism detection.
    - NEGATIVE (faithful/correct): context + question + correct_choice text.
      The correct conclusion should NOT be flagged.

    Verdict: combined (illusions OR contradictions OR paralogisms).
    Paralogism detection is the primary expected signal — wrong choices in
    valid propositional logic arguments are often affirm-the-consequent errors.
    """
    lb_dir = downloads_dir / "logicbench"
    all_rows = []

    for fname in _LOGICBENCH_FILES:
        fpath = lb_dir / fname
        if not fpath.exists():
            continue
        with open(fpath, encoding="utf-8") as fh:
            data = json.load(fh)
        logic_type = data.get("axiom", fname.replace(".json", ""))
        for sample in data.get("samples", []):
            sample["_logic_type"] = logic_type
            all_rows.append(sample)

    if not all_rows:
        raise SystemExit(f"No LogicBench JSON files found in {lb_dir}")

    positives = []
    negatives = []

    for row in all_rows:
        context = text_value(row.get("context", ""))
        question = text_value(row.get("question", ""))
        choices_raw = row.get("choices", {})
        answer_key = text_value(row.get("answer", ""))
        logic_type = row.get("_logic_type", "propositional")

        if not choices_raw:
            continue

        if isinstance(choices_raw, dict):
            choices_list = list(choices_raw.values())
            correct_text = text_value(choices_raw.get(answer_key, ""))
            if not correct_text and choices_list:
                correct_text = text_value(choices_list[0])
        else:
            choices_list = list(choices_raw)
            idx = ord(answer_key.upper()) - ord("A") if len(answer_key) == 1 else 0
            correct_text = text_value(choices_list[idx]) if 0 <= idx < len(choices_list) else ""

        if not correct_text:
            continue

        neg_text = f"Context: {context}\nQuestion: {question}\nConclusion: {correct_text}"
        negatives.append((truncate_text(neg_text), False, logic_type))

        for choice_val in choices_list:
            wrong_text = text_value(choice_val)
            if wrong_text and wrong_text != correct_text:
                pos_text = f"Context: {context}\nQuestion: {question}\nConclusion: {wrong_text}"
                positives.append((truncate_text(pos_text), True, logic_type))

    rng = random.Random(seed)
    pos_sample = rng.sample(positives, min(n_per_class, len(positives)))
    neg_sample = rng.sample(negatives, min(n_per_class, len(negatives)))
    pairs = pos_sample + neg_sample
    rng.shuffle(pairs)

    n = len(pos_sample)
    print(f"\n{'=' * 60}\nOFFICIAL BENCHMARK: LogicBench (Propositional)\n{'=' * 60}")
    print(f"  Dataset: {lb_dir}  ({len(all_rows)} samples, {len(_LOGICBENCH_FILES)} logic types)")
    print(f"  Running on {len(pairs)} pairs ({n} wrong + {len(neg_sample)} correct)...")
    print("  Mode: S38v2 entity | S43 axiom | S46 pronoun+structural+vocab+disjunctive")

    def run_pair(pair) -> tuple[bool, bool]:
        text, is_issue, logic_type = pair
        try:
            verdict = run_pipeline_heuristic(text)
            predicted = (
                verdict.get("has_illusions", False)
                or verdict.get("has_contradictions", False)
                or verdict.get("has_paralogisms", False)
                or logicbench_entity_oracle(text)
                or logicbench_axiom_oracle(text, logic_type)
                or logicbench_pronoun_oracle(text)
                or logicbench_structural_oracle(text, logic_type)
                or logicbench_vocab_oracle(text, logic_type)
                or logicbench_disjunctive_oracle(text, logic_type)
            )
            return is_issue, predicted
        except BenchmarkExecutionError:
            return is_issue, False

    with ThreadPoolExecutor(max_workers=workers) as pool:
        all_results = list(pool.map(run_pair, pairs))

    eval_results = [
        {
            "text_preview": text[:100],
            "ground_truth": gt,
            "predicted": pred,
            "category": cat,
            "risk": "unknown",
            "error": None,
        }
        for (text, _, cat), (gt, pred) in zip(pairs, all_results)
    ]
    return package_result(eval_results, "LogicBench propositional", str(lb_dir), n)
