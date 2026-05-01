#!/usr/bin/env python3
"""
validate_priors.py — PureReason community prior atlas validator.

Validates data/priors.yaml for:
  - Required fields and correct types
  - Duplicate IDs
  - Empty signal lists
  - Confidence out of range
  - Unreachable URLs (optional, with --check-sources)

Usage:
  python3 scripts/validate_priors.py
  python3 scripts/validate_priors.py --check-sources
  python3 scripts/validate_priors.py --priors path/to/other.yaml
"""

import argparse
import sys
import urllib.error
import urllib.request
from pathlib import Path

try:
    import yaml
except ImportError:
    print("ERROR: PyYAML not installed. Run: pip install pyyaml")
    sys.exit(1)

REQUIRED_FIELDS = {
    "id": str,
    "category": str,
    "topic_signals": list,
    "myth_signals": list,
    "correction_signals": list,
    "confidence": (int, float),
}

# Optional but recommended — warn if missing/empty
RECOMMENDED_FIELDS = {"claim": str, "correct": str, "source": str}

VALID_CATEGORIES = {
    "science",
    "history",
    "geography",
    "medicine",
    "technology",
    "law",
    "economics",
    "culture",
    "mathematics",
    "language",
    "environment",
    "politics",
    "psychology",
    "art",
    "religion",
    "nutrition",
    "space",
    "biology",
    "physics",
    "chemistry",
}


def validate_prior(prior: dict, idx: int) -> tuple[list[str], list[str]]:
    errors = []
    warnings = []
    ref = f"entry[{idx}] (id={prior.get('id', '<missing>')})"

    for field, expected_type in REQUIRED_FIELDS.items():
        if field not in prior:
            errors.append(f"{ref}: missing required field '{field}'")
            continue
        value = prior[field]
        if not isinstance(value, expected_type):
            errors.append(
                f"{ref}: field '{field}' must be {expected_type}, got {type(value).__name__}"
            )

    for field, _expected_type in RECOMMENDED_FIELDS.items():
        value = prior.get(field)
        if value is None or (isinstance(value, str) and not value.strip()):
            warnings.append(f"{ref}: recommended field '{field}' is missing or empty")

    # Validate ID format
    pid = prior.get("id", "")
    if pid and not all(c.isalnum() or c == "_" for c in pid):
        errors.append(f"{ref}: id must be snake_case alphanumeric (got '{pid}')")

    # Validate category
    cat = prior.get("category", "")
    if cat and cat not in VALID_CATEGORIES:
        errors.append(
            f"{ref}: unknown category '{cat}'. "
            f"Valid categories: {', '.join(sorted(VALID_CATEGORIES))}"
        )

    # Validate signal lists are non-empty
    for field in ("topic_signals", "myth_signals", "correction_signals"):
        val = prior.get(field)
        if isinstance(val, list) and len(val) == 0:
            errors.append(f"{ref}: '{field}' must not be empty")
        if isinstance(val, list):
            for item in val:
                if not isinstance(item, str):
                    errors.append(f"{ref}: '{field}' items must be strings")
                elif not item.strip():
                    errors.append(f"{ref}: '{field}' contains an empty string")

    # Validate confidence range
    conf = prior.get("confidence")
    if isinstance(conf, (int, float)) and not (0.0 <= conf <= 1.0):
        errors.append(f"{ref}: confidence must be 0.0–1.0, got {conf}")

    # Validate claim / correct are non-trivial
    for field in ("claim", "correct"):
        val = prior.get(field, "")
        if isinstance(val, str) and len(val.strip()) < 10:
            errors.append(f"{ref}: '{field}' is suspiciously short (< 10 chars)")

    return errors, warnings


def check_source(url: str, timeout: int = 5) -> str | None:
    """Return an error string if the URL is unreachable, else None."""
    try:
        req = urllib.request.Request(url, method="HEAD")
        with urllib.request.urlopen(req, timeout=timeout):
            return None
    except (urllib.error.URLError, Exception) as e:
        return str(e)


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate data/priors.yaml")
    parser.add_argument(
        "--priors",
        default=str(Path(__file__).parent.parent / "data" / "priors.yaml"),
        help="Path to priors YAML file (default: data/priors.yaml)",
    )
    parser.add_argument(
        "--check-sources",
        action="store_true",
        help="Perform HTTP HEAD requests to verify source URLs",
    )
    args = parser.parse_args()

    priors_path = Path(args.priors)
    if not priors_path.exists():
        print(f"ERROR: File not found: {priors_path}")
        return 1

    with open(priors_path, encoding="utf-8") as f:
        try:
            data = yaml.safe_load(f)
        except yaml.YAMLError as e:
            print(f"ERROR: YAML parse error: {e}")
            return 1

    if not isinstance(data, list):
        print("ERROR: priors.yaml must be a list of prior objects at the top level")
        return 1

    all_errors: list[str] = []
    all_warnings: list[str] = []
    seen_ids: dict[str, int] = {}
    warning_count = 0

    for idx, prior in enumerate(data):
        if not isinstance(prior, dict):
            all_errors.append(f"entry[{idx}]: must be a mapping, got {type(prior).__name__}")
            continue

        errors, warnings = validate_prior(prior, idx)
        all_errors.extend(errors)
        all_warnings.extend(warnings)

        pid = prior.get("id", "")
        if pid:
            if pid in seen_ids:
                all_errors.append(
                    f"entry[{idx}]: duplicate id '{pid}' (first seen at entry[{seen_ids[pid]}])"
                )
            else:
                seen_ids[pid] = idx

        if args.check_sources:
            source = prior.get("source", "")
            if source and isinstance(source, str):
                err = check_source(source)
                if err:
                    msg = f"entry[{idx}] (id={pid}): source URL unreachable: {source} ({err})"
                    print(f"  WARN  {msg}")
                    warning_count += 1

    if all_errors:
        print(f"\nValidation FAILED — {len(all_errors)} error(s):\n")
        for err in all_errors:
            print(f"  ERROR  {err}")
        print()
        return 1
    else:
        if all_warnings:
            print(
                f"  WARN  {len(all_warnings)} entries missing recommended fields (claim/correct/source)"
            )
        prior_count = len(data)
        id_count = len(seen_ids)
        print(f"OK — {prior_count} priors validated ({id_count} unique IDs)", end="")
        if warning_count:
            print(f", {warning_count} source warning(s)")
        else:
            print()
        return 0


if __name__ == "__main__":
    sys.exit(main())
