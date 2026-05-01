#!/usr/bin/env python3
"""
Process Wikipedia XML dump to JSONL corpus format.

Usage:
    python3 scripts/process_wikipedia_corpus.py \
        --input enwiki-latest-abstract.xml \
        --output data/corpus/wikipedia_v1.0.jsonl \
        --spacy-model en_core_web_sm
"""

import argparse
import json
import re
import xml.etree.ElementTree as ET
from datetime import datetime

try:
    import spacy

    SPACY_AVAILABLE = True
except ImportError:
    SPACY_AVAILABLE = False
    print("Warning: spaCy not available. Entity extraction will be disabled.")


def extract_entities(text: str, nlp) -> list[str]:
    """Extract named entities from text using spaCy."""
    if not SPACY_AVAILABLE or nlp is None:
        return []

    doc = nlp(text)
    entities = []
    for ent in doc.ents:
        if ent.label_ in ["PERSON", "ORG", "GPE", "LOC", "EVENT", "WORK_OF_ART"]:
            entities.append(ent.text)

    # Deduplicate while preserving order
    seen = set()
    return [e for e in entities if not (e in seen or seen.add(e))]


def clean_abstract(text: str) -> str:
    """Clean Wikipedia abstract text."""
    # Remove Wikipedia markup
    text = re.sub(r"<[^>]+>", "", text)  # Remove HTML tags
    text = re.sub(r"\[\[([^\]]+)\]\]", r"\1", text)  # Remove wiki links
    text = re.sub(r"\{\{[^\}]+\}\}", "", text)  # Remove templates

    # Normalize whitespace
    text = " ".join(text.split())

    return text.strip()


def parse_wikipedia_xml(xml_file: str, output_file: str, spacy_model: str = "en_core_web_sm"):
    """
    Parse Wikipedia XML dump and convert to JSONL format.

    Args:
        xml_file: Path to enwiki-latest-abstract.xml
        output_file: Path to output JSONL file
        spacy_model: spaCy model name for entity extraction
    """
    # Load spaCy model
    nlp = None
    if SPACY_AVAILABLE:
        try:
            nlp = spacy.load(spacy_model)
            print(f"Loaded spaCy model: {spacy_model}")
        except OSError:
            print(
                f"Warning: spaCy model '{spacy_model}' not found. Run: python -m spacy download {spacy_model}"
            )
            print("Entity extraction will be disabled.")

    print(f"Processing {xml_file}...")
    article_count = 0

    with open(output_file, "w", encoding="utf-8") as out_f:
        # Parse XML iteratively (memory-efficient)
        context = ET.iterparse(xml_file, events=("start", "end"))
        context = iter(context)
        event, root = next(context)

        current_doc = {}
        in_doc = False

        for event, elem in context:
            if event == "start" and elem.tag == "doc":
                in_doc = True
                current_doc = {}

            elif event == "end" and in_doc:
                if elem.tag == "title":
                    current_doc["title"] = elem.text or ""
                elif elem.tag == "url":
                    current_doc["url"] = elem.text or ""
                elif elem.tag == "abstract":
                    abstract = elem.text or ""
                    current_doc["abstract"] = clean_abstract(abstract)
                elif elem.tag == "links":
                    # Extract categories from links
                    sublinks = elem.findall(".//sublink")
                    categories = []
                    for sublink in sublinks:
                        if sublink.text and "Category:" in sublink.text:
                            category = sublink.text.replace("Category:", "").strip()
                            categories.append(category)
                    current_doc["categories"] = categories

                elif elem.tag == "doc":
                    # Document complete
                    in_doc = False

                    # Extract ID from URL
                    url = current_doc.get("url", "")
                    doc_id = url.split("=")[-1] if "=" in url else str(article_count)

                    # Skip if missing required fields
                    if not current_doc.get("title") or not current_doc.get("abstract"):
                        root.clear()
                        continue

                    # Extract entities
                    entities = extract_entities(current_doc["abstract"], nlp)

                    # Word count
                    word_count = len(current_doc["abstract"].split())

                    # Build record
                    record = {
                        "id": doc_id,
                        "title": current_doc["title"],
                        "abstract": current_doc["abstract"],
                        "url": current_doc.get("url", ""),
                        "categories": current_doc.get("categories", []),
                        "entities": entities,
                        "last_modified": datetime.utcnow().isoformat() + "Z",
                        "word_count": word_count,
                    }

                    # Write JSONL
                    out_f.write(json.dumps(record, ensure_ascii=False) + "\n")
                    article_count += 1

                    if article_count % 10000 == 0:
                        print(f"Processed {article_count:,} articles...")

                    # Clear root to free memory
                    root.clear()

    print(f"\nCompleted! Processed {article_count:,} articles.")
    print(f"Output: {output_file}")


def main():
    parser = argparse.ArgumentParser(description="Process Wikipedia XML dump to JSONL")
    parser.add_argument("--input", required=True, help="Path to enwiki-latest-abstract.xml")
    parser.add_argument("--output", required=True, help="Path to output JSONL file")
    parser.add_argument(
        "--spacy-model", default="en_core_web_sm", help="spaCy model for entity extraction"
    )

    args = parser.parse_args()

    parse_wikipedia_xml(args.input, args.output, args.spacy_model)


if __name__ == "__main__":
    main()
