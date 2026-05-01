"""FELM benchmark (factual, commonsense, scientific, math domains)."""

from pathlib import Path

from .core import (
    BenchmarkExecutionError,
    load_json_records,
    run_pipeline_claims,
    text_value,
)
from .felm_oracles import _arithmetic_error_in_felm, _word_problem_error_in_felm
from .metrics import evaluate_pairs_combined, package_result, sample_balanced
from .semantic import _batch_felm_semantic_scores, _get_st_model
from .verdicts import verdict_is_issue_felm


def benchmark_felm(downloads_dir: Path, n_per_class: int, seed: int, workers: int) -> dict:
    path = downloads_dir / "felm" / "all.jsonl"
    rows = load_json_records(path)

    def make_pair(row: dict):
        prompt = text_value(row.get("prompt"))
        response = text_value(row.get("response"))
        labels = row.get("labels", [])
        if not response:
            return None
        has_issue = any(label is False for label in labels)
        text = f"Prompt: {prompt}\nAnswer: {response}" if prompt else response
        return (text, has_issue, row.get("domain", "felm"))

    pairs = sample_balanced(rows, make_pair, n_per_class, seed)
    n = len(pairs) // 2

    print(f"\n{'=' * 60}\nOFFICIAL BENCHMARK: FELM\n{'=' * 60}")
    print(f"  Dataset: {path}")
    print(f"  Running on {len(pairs)} samples ({n} factual + {n} non-factual)...")

    st_model = _get_st_model()
    if st_model is not None:
        print("  Mode: S45 semantic divergence (threshold=0.86) + arithmetic + heuristic")
        print(f"  Pre-computing prompt-response cosine for {len(pairs)} pairs...")
        felm_semantic_flags = _batch_felm_semantic_scores(pairs, st_model, threshold=0.86)
    else:
        print("  Mode: FELM combined: heuristic (world priors/illusions) + arithmetic verifier")
        print("        + claims segmentation (TRIZ P1 OR rule — any risky claim = ISSUE)")
        felm_semantic_flags = {}

    def felm_combined_verdict(text: str, verdict: dict) -> bool:
        """OR-combine heuristic verdict with arithmetic + claims segmentation + S45 semantic.

        Track 1 (S26): word-problem arithmetic extraction (_word_problem_error_in_felm).
        Track 2: explicit expression arithmetic (_arithmetic_error_in_felm).
        Track 3 (TRIZ P1 Segmentation): per-claim OR rule via `pure-reason claims`.
          Any risky claim (illusion OR antinomy) in a multi-sentence response flags ISSUE.
        Track 4 (heuristic): world priors + illusions + paralogisms (verdict_is_issue_felm).
        Track 5 (S45): semantic cosine divergence — prompt vs response.
        """
        if verdict_is_issue_felm(verdict):
            return True
        if _arithmetic_error_in_felm(text) or _word_problem_error_in_felm(text):
            return True
        if felm_semantic_flags.get(text[:100], False):
            return True
        try:
            claims_verdict = run_pipeline_claims(text)
            if claims_verdict.get("has_illusions") or claims_verdict.get("has_contradictions"):
                return True
        except BenchmarkExecutionError:
            pass
        return False

    results = evaluate_pairs_combined(pairs, workers, felm_combined_verdict)
    return package_result(results, "FELM official", str(path), n)
