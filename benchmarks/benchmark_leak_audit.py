#!/usr/bin/env python3
"""TRIZ-42 NE-1 / NE-2 guard: detect benchmark → world-prior leakage.

Reads the signal strings (``topic_signals``, ``myth_signals``,
``correction_signals``) from ``data/misconceptions_corpus_v1.jsonl``
and cross-checks them against every downloaded benchmark's test CSV/JSONL.
If any signal string appears verbatim (or near-verbatim by trigram Jaccard)
in a benchmark's ground-truth text, the audit fails.

The intent is to make it *structurally impossible* to add world-prior entries
lifted from test data. Run this in CI; fail the build on any hit.

Usage::

    python3 benchmarks/benchmark_leak_audit.py
    python3 benchmarks/benchmark_leak_audit.py --jaccard 0.25 --verbose

Exit codes
----------
  0  No leakage detected.
  1  One or more signal strings overlap with benchmark test data.
  2  Inputs missing (misconceptions_corpus_v1.jsonl or benchmark files not found).
"""

from __future__ import annotations

import argparse
import csv
import json
import re
import sys
from collections.abc import Iterator
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
WORLD_PRIORS_JSONL = REPO / "data" / "misconceptions_corpus_v1.jsonl"
DOWNLOADS = REPO / "benchmarks" / "downloads"


# ─── Signal extraction from misconceptions_corpus_v1.jsonl ────────────────────

def extract_signals(path: Path) -> list[tuple[str, str]]:
    """Return list of (kind, signal) tuples from JSONL corpus."""
    if not path.exists():
        return []
    
    out: list[tuple[str, str]] = []
    try:
        with open(path, encoding="utf-8") as fh:
            for line in fh:
                line = line.strip()
                if not line:
                    continue
                try:
                    row = json.loads(line)
                except json.JSONDecodeError:
                    continue
                
                # Extract signals from topic_signals, myth_signals, correction_signals
                for kind in ("topic_signals", "myth_signals", "correction_signals"):
                    signals = row.get(kind, [])
                    if isinstance(signals, list):
                        for sig in signals:
                            if isinstance(sig, str):
                                sig = sig.strip().lower()
                                if len(sig) >= 8:  # skip ultra-short keyword fragments
                                    out.append((kind, sig))
    except OSError:
        pass
    
    return out



# ─── Benchmark text extraction ───────────────────────────────────────────────


def _truthfulqa_texts() -> Iterator[tuple[str, str]]:
    path = DOWNLOADS / "truthfulqa" / "TruthfulQA.csv"
    if not path.exists():
        return
    with open(path, newline="", encoding="utf-8") as fh:
        for row in csv.DictReader(fh):
            q = row.get("Question", "")
            best = row.get("Best Answer", "")
            incorrect = row.get("Incorrect Answers", "")
            for field, val in (("Question", q), ("Best Answer", best)):
                if val:
                    yield (f"truthfulqa/{field}", val)
            for seg in incorrect.split(";"):
                seg = seg.strip()
                if seg:
                    yield ("truthfulqa/Incorrect Answer", seg)


def _halueval_texts() -> Iterator[tuple[str, str]]:
    for name in ("qa_data.json", "dialogue_data.json"):
        path = DOWNLOADS / "halueval" / name
        if not path.exists():
            continue
        try:
            with open(path, encoding="utf-8") as fh:
                for line in fh:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        row = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    for key in (
                        "right_answer",
                        "hallucinated_answer",
                        "right_response",
                        "hallucinated_response",
                        "question",
                        "knowledge",
                    ):
                        val = row.get(key)
                        if isinstance(val, str) and val:
                            yield (f"halueval/{name}/{key}", val)
        except OSError:
            continue


def _felm_texts() -> Iterator[tuple[str, str]]:
    path = DOWNLOADS / "felm" / "all.jsonl"
    if not path.exists():
        return
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                continue
            for key in ("response", "prompt"):
                val = row.get(key)
                if isinstance(val, str) and val:
                    yield (f"felm/{key}", val)


BENCHMARK_SOURCES = (
    ("truthfulqa", _truthfulqa_texts),
    ("halueval", _halueval_texts),
    ("felm", _felm_texts),
)


# ─── Matching ────────────────────────────────────────────────────────────────


def _normalize(s: str) -> str:
    return re.sub(r"\s+", " ", s.lower()).strip()


def _trigrams(s: str) -> set[tuple[str, str, str]]:
    toks = _normalize(s).split()
    if len(toks) < 3:
        return set()
    return set(zip(toks, toks[1:], toks[2:]))


def _jaccard(a: set, b: set) -> float:
    if not a or not b:
        return 0.0
    return len(a & b) / len(a | b)


def find_hits(
    signals: list[tuple[str, str]],
    jaccard_threshold: float,
) -> list[dict]:
    hits: list[dict] = []
    signal_trigrams = [(kind, sig, _trigrams(sig)) for kind, sig in signals]

    for bench, source in BENCHMARK_SOURCES:
        for label, text in source():
            norm = _normalize(text)
            text_tgs = _trigrams(text)
            for kind, sig, sig_tgs in signal_trigrams:
                if sig in norm:
                    hits.append(
                        {
                            "benchmark": bench,
                            "field": label,
                            "signal_kind": kind,
                            "signal": sig,
                            "match": "verbatim_substring",
                            "sample": text[:240],
                        }
                    )
                    continue
                if not sig_tgs or not text_tgs:
                    continue
                j = _jaccard(sig_tgs, text_tgs)
                if j >= jaccard_threshold:
                    hits.append(
                        {
                            "benchmark": bench,
                            "field": label,
                            "signal_kind": kind,
                            "signal": sig,
                            "match": f"trigram_jaccard={j:.2f}",
                            "sample": text[:240],
                        }
                    )
    return hits


# ─── Entry point ─────────────────────────────────────────────────────────────


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument(
        "--jaccard",
        type=float,
        default=0.50,
        help="Trigram Jaccard threshold for soft match (default: 0.50).",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Print per-signal match details.",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Optional JSON path for the hit list.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    if not WORLD_PRIORS_JSONL.exists():
        print(f"ERROR: {WORLD_PRIORS_JSONL} not found", file=sys.stderr)
        return 2

    signals = extract_signals(WORLD_PRIORS_JSONL)
    if not signals:
        print("ERROR: no signals extracted — JSONL may be empty or malformed", file=sys.stderr)
        return 2

    print(f"Extracted {len(signals)} signals from {WORLD_PRIORS_JSONL.name}")
    hits = find_hits(signals, args.jaccard)

    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(hits, indent=2), encoding="utf-8")

    if not hits:
        print("PASS: no overlap between world_priors signals and benchmark test data.")
        return 0

    print(f"FAIL: {len(hits)} signal/benchmark overlaps detected.")
    if args.verbose:
        for h in hits[:50]:
            print(
                f"  [{h['benchmark']}] {h['signal_kind']} signal={h['signal']!r} "
                f"match={h['match']}  sample={h['sample']!r}"
            )
        if len(hits) > 50:
            print(f"  ... {len(hits) - 50} more (see --output json for full list)")
    else:
        # Summary counts by benchmark
        by_bench: dict[str, int] = {}
        for h in hits:
            by_bench[h["benchmark"]] = by_bench.get(h["benchmark"], 0) + 1
        for bench, count in sorted(by_bench.items()):
            print(f"  {bench}: {count} overlap(s)")
        print("Rerun with --verbose to see individual hits.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
