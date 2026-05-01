"""``pureason-check`` — command-line entry point for the Python wrapper.

Usage::

    pureason-check calibrate "The patient must have cancer."
    pureason-check analyze  "The Earth orbits the Moon." --reference "The Earth orbits the Sun."
    pureason-check flags    "Some statement here."
"""

from __future__ import annotations

import argparse
import json
import sys

from . import analyze, calibrate, flags


def _print_json(obj: object) -> None:
    import dataclasses

    print(
        json.dumps(
            dataclasses.asdict(obj) if hasattr(obj, "__dataclass_fields__") else obj, indent=2
        )
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="pureason-check",
        description="PureReason epistemic calibration — Python wrapper",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    # calibrate
    cal_p = sub.add_parser("calibrate", help="Compute ECS for a text")
    cal_p.add_argument("text", help="Text to evaluate")

    # analyze
    ana_p = sub.add_parser("analyze", help="Full analysis with optional hallucination check")
    ana_p.add_argument("text", nargs="?", help="Text to evaluate (omit to read from stdin)")
    ana_p.add_argument("--reference", "-r", help="Reference/knowledge document")
    ana_p.add_argument("--question", "-q", help="Optional question")

    # flags
    fl_p = sub.add_parser("flags", help="Print only the flag list")
    fl_p.add_argument("text", help="Text to evaluate")
    fl_p.add_argument("--reference", "-r", help="Reference/knowledge document")

    args = parser.parse_args(argv)

    try:
        if args.command == "calibrate":
            _print_json(calibrate(args.text))
        elif args.command == "analyze":
            text = args.text if args.text else sys.stdin.read()
            _print_json(analyze(text, reference=args.reference, question=args.question))
        elif args.command == "flags":
            result = flags(args.text, reference=args.reference)
            print(json.dumps(result, indent=2))
    except RuntimeError as exc:
        print(f"Error: {exc}", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
