"""Binary finder and subprocess runner for the pureason package."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
from pathlib import Path
from typing import Any

_BINARY_NAME = "pure-reason"
_BINARY_ENV = "PUREASON_BINARY"

# Candidate paths relative to this file (for dev-mode installs inside the repo)
_RELATIVE_CANDIDATES = [
    Path(__file__).parent.parent / "target" / "release" / _BINARY_NAME,
]


def _find_binary() -> str:
    """Return the path to the pure-reason binary.

    Resolution order:
    1. PUREASON_BINARY environment variable
    2. ./target/release/pure-reason relative to this package (dev mode in repo)
    3. PATH lookup

    Raises RuntimeError if none are found.
    """
    env_override = os.environ.get(_BINARY_ENV)
    if env_override:
        p = Path(env_override)
        if p.is_file():
            return str(p)
        raise RuntimeError(f"{_BINARY_ENV}={env_override!r} is set but the file does not exist.")

    # Prefer the local repo build over any system-wide PATH install, since the
    # PATH version may be an older release.
    for candidate in _RELATIVE_CANDIDATES:
        if candidate.is_file():
            return str(candidate)

    in_path = shutil.which(_BINARY_NAME)
    if in_path:
        return in_path

    raise RuntimeError(
        f"Could not find the '{_BINARY_NAME}' binary.\n"
        "Options:\n"
        "  1. Build it:    cargo build --release  (then run from repo root)\n"
        "  2. Install it:  cargo install --path crates/pure-reason-cli\n"
        f"  3. Set env var: {_BINARY_ENV}=/path/to/pure-reason"
    )


def _run(args: list[str], stdin_text: str | None = None) -> dict[str, Any]:
    """Run pure-reason with *args*, parse JSON output, return parsed dict."""
    binary = _find_binary()
    cmd = [binary, *args, "--format", "json"]

    try:
        result = subprocess.run(
            cmd,
            input=stdin_text.encode() if stdin_text else None,
            capture_output=True,
            timeout=30,
        )
    except FileNotFoundError as exc:
        raise RuntimeError(f"Binary not found at: {binary}") from exc
    except subprocess.TimeoutExpired as exc:
        raise RuntimeError("pure-reason timed out after 30 s") from exc

    raw = result.stdout.decode("utf-8", errors="replace").strip()
    if not raw:
        stderr = result.stderr.decode("utf-8", errors="replace").strip()
        raise RuntimeError(
            f"pure-reason returned empty output (exit={result.returncode}).\n{stderr}"
        )

    try:
        return json.loads(raw)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"pure-reason output is not valid JSON:\n{raw[:500]}") from exc
