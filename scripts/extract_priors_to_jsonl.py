#!/usr/bin/env python3
"""Extract misconception priors from Rust code to JSON lines."""

import json
import re
from pathlib import Path

rust_file = Path("crates/pure-reason-core/src/world_priors.rs")
output_file = Path("data/misconceptions_corpus_v1.jsonl")

# Read the Rust file
content = rust_file.read_text()

# Extract all MisconceptionPrior blocks
prior_pattern = r"MisconceptionPrior\s*\{([^}]+)\}"
priors_data = []

for match in re.finditer(prior_pattern, content, re.DOTALL):
    block = match.group(1)

    # Extract id
    id_match = re.search(r'id:\s*"([^"]+)"', block)
    if not id_match:
        continue
    prior_id = id_match.group(1)

    # Extract arrays
    def extract_array(field_name: str) -> list[str]:
        pattern = rf"{field_name}:\s*&\[(.*?)\]"
        m = re.search(pattern, block, re.DOTALL)
        if not m:
            return []
        items = re.findall(r'"([^"]+)"', m.group(1))
        return items

    topic_signals = extract_array("topic_signals")
    myth_signals = extract_array("myth_signals")
    correction_signals = extract_array("correction_signals")

    priors_data.append(
        {
            "id": prior_id,
            "topic_signals": topic_signals,
            "myth_signals": myth_signals,
            "correction_signals": correction_signals,
        }
    )

# Write JSONL
output_file.parent.mkdir(parents=True, exist_ok=True)
with open(output_file, "w") as f:
    for prior in priors_data:
        f.write(json.dumps(prior) + "\n")

print(f"Extracted {len(priors_data)} priors to {output_file}")
