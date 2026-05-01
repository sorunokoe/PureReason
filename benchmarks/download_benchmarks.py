#!/usr/bin/env python3
"""
Download official upstream benchmark assets used by PureReason.

By default this script downloads all benchmark files declared in
`benchmarks/benchmark_sources.json` into `benchmarks/downloads/`.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
MANIFEST_PATH = SCRIPT_DIR / "benchmark_sources.json"
DEFAULT_DEST = SCRIPT_DIR / "downloads"


def load_manifest() -> dict:
    with open(MANIFEST_PATH, encoding="utf-8") as handle:
        return json.load(handle)


def parse_args(manifest: dict) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Download official benchmark files used by PureReason."
    )
    parser.add_argument(
        "--benchmarks",
        default="all",
        help=(
            "Comma-separated benchmark keys to download "
            f"(available: {', '.join(sorted(manifest))}) or 'all'."
        ),
    )
    parser.add_argument(
        "--dest",
        default=str(DEFAULT_DEST),
        help="Destination directory for downloaded assets.",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Re-download files even if they already exist.",
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="List available benchmark keys and exit.",
    )
    return parser.parse_args()


def resolve_selection(selection: str, manifest: dict) -> list[str]:
    if selection == "all":
        return list(manifest.keys())
    keys = [item.strip() for item in selection.split(",") if item.strip()]
    unknown = [item for item in keys if item not in manifest]
    if unknown:
        raise SystemExit(
            "Unknown benchmark(s): "
            + ", ".join(sorted(unknown))
            + f". Available: {', '.join(sorted(manifest))}"
        )
    return keys


def download_file(url: str, destination: Path, force: bool) -> str:
    if destination.exists() and not force:
        return "cached"

    destination.parent.mkdir(parents=True, exist_ok=True)
    result = subprocess.run(
        ["curl", "-LfsS", "-A", "PureReason/0.1", "-o", str(destination), url],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise SystemExit(result.stderr.strip() or result.stdout.strip() or f"curl failed for {url}")
    return "downloaded"


def main() -> int:
    manifest = load_manifest()
    args = parse_args(manifest)

    if args.list:
        print("Available benchmarks:")
        for key in sorted(manifest):
            info = manifest[key]
            print(f"  - {key}: {info['display_name']}")
        return 0

    destination_root = Path(args.dest).resolve()
    selected = resolve_selection(args.benchmarks, manifest)

    print(f"Destination: {destination_root}")
    print(f"Benchmarks:  {', '.join(selected)}")

    downloaded = 0
    cached = 0
    for key in selected:
        info = manifest[key]
        print(f"\n== {info['display_name']} ({key}) ==")
        print(info.get("homepage", ""))
        for file_spec in info["files"]:
            target = destination_root / file_spec["path"]
            status = download_file(file_spec["url"], target, args.force)
            if status == "downloaded":
                downloaded += 1
            else:
                cached += 1
            size_kb = target.stat().st_size / 1024 if target.exists() else 0.0
            print(f"  [{status}] {target.relative_to(destination_root)}  ({size_kb:.1f} KiB)")

    print(f"\nDone. downloaded={downloaded} cached={cached} root={destination_root}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
