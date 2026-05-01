#!/usr/bin/env python3
"""Source misconceptions from Wikipedia misconception articles.

This script fetches misconception articles from Wikipedia and extracts individual
misconceptions with full provenance (Wikipedia URL, section, date).

TRIZ-42 NE-2 remedy: Replace TruthfulQA-derived misconceptions with
independent external source (Wikipedia) to break data leakage.

Usage:
    python3 scripts/source_wikipedia_misconceptions.py
    python3 scripts/source_wikipedia_misconceptions.py --output custom_output.jsonl

Output:
    misconceptions_corpus_v2_wikipedia.jsonl (by default)
    Each record: {
        "id": "unique_id",
        "topic_signals": [...],
        "myth_signals": [...],
        "correction_signals": [...],
        "source": "Wikipedia",
        "article": "List of common misconceptions about...",
        "fetched_date": "2026-04-28",
        "url": "https://en.wikipedia.org/wiki/..."
    }
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime
from pathlib import Path

try:
    import wikipediaapi
except ImportError:
    print("ERROR: wikipediaapi not installed. Run: pip3 install wikipedia-api")
    sys.exit(2)


def extract_keywords(text: str, max_keywords: int = 5) -> list[str]:
    """Extract meaningful keywords from text."""
    # Remove citations and extra whitespace
    text = re.sub(r'\[\d+\]', '', text)
    text = re.sub(r'\[\w+[^\]]*\]', '', text)
    text = re.sub(r'\s+', ' ', text).strip()

    if len(text) < 8:
        return []

    # Extract phrases/chunks separated by punctuation
    phrases = re.split(r'[,.;:\-–—?!]', text)

    keywords = []
    for phrase in phrases:
        phrase = phrase.strip().lower()
        # Keep phrases that are 8-100 chars, 2-10 words
        word_count = len(phrase.split())
        if 8 <= len(phrase) <= 100 and 2 <= word_count <= 10:
            keywords.append(phrase)

    return keywords[:max_keywords]


def parse_history_misconceptions(text: str, article_title: str) -> list[dict]:
    """Parse Wikipedia history misconceptions article."""
    records = []
    record_id = 0

    # Split by double newlines to get paragraphs
    paragraphs = re.split(r'\n\n+', text)

    current_section = "general"
    for para in paragraphs:
        para = para.strip()
        if not para:
            continue

        # Check if this is a section header (bold text)
        if para.startswith("==") or len(para) < 30:
            # Try to extract section name
            match = re.search(r'==+\s*([^=]+)\s*==+', para)
            if match:
                current_section = match.group(1).strip().lower()
            continue

        # This should be a misconception entry
        # Format: "Correction statement about X. More details about Y."

        lines = para.split('\n')
        first_line = lines[0] if lines else ""

        if len(first_line) < 20:
            continue

        # Extract myth signals (first part of correction, usually includes what is FALSE)
        # Example: "The Pyramids were not constructed with slave labor."
        # Extract: "pyramids", "slave labor", "constructed"
        myth_signals = extract_keywords(first_line, max_keywords=3)

        # Extract correction signals (usually indicated by "was not", "did not", "were not")
        correction_signals = []
        if "not" in first_line.lower():
            correction_signals.append("not " + re.sub(r'.*was not\s+', '', first_line, flags=re.IGNORECASE)[:50])

        # Get remaining details for additional signals
        for line in lines[1:2]:
            if line.strip():
                signals = extract_keywords(line, max_keywords=2)
                correction_signals.extend(signals)

        # Clean up correction signals
        correction_signals = [s for s in correction_signals if len(s) > 5][:3]

        if not myth_signals:
            continue

        record = {
            "id": f"wiki_hist_{record_id:04d}",
            "topic_signals": [current_section, "history", "misconception"],
            "myth_signals": myth_signals,
            "correction_signals": correction_signals,
            "source": "Wikipedia",
            "article": article_title,
            "fetched_date": datetime.now().strftime("%Y-%m-%d"),
            "url": f"https://en.wikipedia.org/wiki/{article_title.replace(' ', '_')}"
        }

        records.append(record)
        record_id += 1

    return records


def fetch_wikipedia_misconceptions() -> list[dict]:
    """Fetch misconceptions from Wikipedia articles."""
    wiki = wikipediaapi.Wikipedia(
        language='en',
        user_agent='PureReason-TRIZ-NE2-Remedy/1.0 (github.com/sorunokoe/PureReason)'
    )

    # These Wikipedia articles exist and contain actual misconceptions
    articles = [
        ("List of common misconceptions about history", parse_history_misconceptions),
    ]

    records = []

    for article_title, parser_func in articles:
        print(f"  Fetching: {article_title}")
        page = wiki.page(article_title)

        if not page.exists():
            print("    → Not found, skipping")
            continue

        text = page.text
        if not text or len(text) < 100:
            print("    → Empty, skipping")
            continue

        parsed = parser_func(text, article_title)
        records.extend(parsed)
        print(f"    → Extracted {len(parsed)} records")

    return records


def save_corpus(records: list[dict], output_path: Path) -> None:
    """Save records as JSONL corpus file."""
    with open(output_path, 'w', encoding='utf-8') as fh:
        for record in records:
            fh.write(json.dumps(record) + '\n')
    print(f"✓ Saved {len(records)} records to {output_path}")


def main():
    parser = argparse.ArgumentParser(
        description="Source misconceptions from Wikipedia"
    )
    parser.add_argument(
        '--output',
        type=Path,
        default=Path(__file__).parent.parent / 'data' / 'misconceptions_corpus_v2_wikipedia.jsonl',
        help='Output JSONL file path'
    )
    args = parser.parse_args()

    print("Fetching Wikipedia misconception articles...")
    records = fetch_wikipedia_misconceptions()

    if not records:
        print("ERROR: No records fetched", file=sys.stderr)
        sys.exit(1)

    print(f"Fetched {len(records)} misconception records")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    save_corpus(records, args.output)
    print(f"External corpus v2 ready: {args.output}")


if __name__ == '__main__':
    main()


