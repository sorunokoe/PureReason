"""Shared constants, error types, and CLI runner utilities."""

import json
import os
import subprocess
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent.parent
CLI_BIN = REPO / "target" / "release" / "pure-reason"
BENCHMARKS_DIR = REPO / "benchmarks"
DOWNLOADS_DIR = BENCHMARKS_DIR / "downloads"
RESULTS_DIR = BENCHMARKS_DIR / "results"
RESULTS_DIR.mkdir(exist_ok=True)
MAX_CONTEXT_CHARS = 6000

# S53 holdout protocol flag — set by main() when --holdout is passed
_HOLDOUT_MODE: bool = False


class BenchmarkExecutionError(RuntimeError):
    """Raised when a benchmark sample cannot be evaluated faithfully."""


def ensure_cli() -> None:
    if CLI_BIN.exists():
        return
    print("Building release CLI binary...")
    result = subprocess.run(
        ["cargo", "build", "-p", "pure-reason-cli", "--release"],
        cwd=REPO,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise SystemExit(result.stderr or result.stdout or "cargo build failed")


def truncate_text(text: str, limit: int = MAX_CONTEXT_CHARS) -> str:
    if len(text) <= limit:
        return text
    return text[:limit].rstrip() + "\n[context truncated for benchmark runtime]"


def text_value(value) -> str:
    if value is None:
        return ""
    if isinstance(value, str):
        return value.strip()
    return json.dumps(value, ensure_ascii=False).strip()


def load_json_records(path: Path) -> list[dict]:
    with open(path, encoding="utf-8") as handle:
        sample = handle.read(1)
        handle.seek(0)
        if sample == "[":
            return json.load(handle)
        return [json.loads(line) for line in handle if line.strip()]


def extract_json_payload(stdout: str) -> str:
    cleaned = stdout.strip()
    if not cleaned:
        return cleaned
    start = cleaned.find("{")
    return cleaned[start:] if start >= 0 else cleaned


def command_label(cmd: list[str]) -> str:
    return " ".join(str(part) for part in cmd)


def run_json_command(
    cmd: list[str],
    *,
    input_text: str,
    timeout: int,
    env: dict[str, str],
) -> dict:
    try:
        result = subprocess.run(
            cmd,
            input=input_text,
            capture_output=True,
            text=True,
            timeout=timeout,
            env=env,
        )
    except Exception as exc:
        raise BenchmarkExecutionError(f"{command_label(cmd)} failed to execute: {exc}") from exc

    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or "no output"
        raise BenchmarkExecutionError(
            f"{command_label(cmd)} exited with code {result.returncode}: {detail}"
        )

    stdout = result.stdout.strip()
    if not stdout:
        raise BenchmarkExecutionError(f"{command_label(cmd)} produced empty output")

    try:
        return json.loads(extract_json_payload(stdout))
    except json.JSONDecodeError as exc:
        preview = stdout[:200].replace("\n", " ")
        raise BenchmarkExecutionError(
            f"{command_label(cmd)} returned invalid JSON: {preview}"
        ) from exc


def run_pipeline_heuristic(text: str) -> dict:
    env = dict(os.environ)
    env.setdefault("RUST_LOG", "error")
    data = run_json_command(
        [str(CLI_BIN), "analyze", "--format", "json"],
        input_text=text,
        timeout=30,
        env=env,
    )
    verdict = data.get("verdict")
    if not isinstance(verdict, dict):
        raise BenchmarkExecutionError("pure-reason analyze returned no verdict object")
    verdict["error"] = None
    return verdict


def run_pipeline_claims(text: str) -> dict:
    """Claim IR pipeline: per-sentence analysis via `pure-reason claims`.

    For FELM-style multi-claim responses, analyzing the whole text as one
    block misses isolated false claims that don't affect the overall ECS.
    Running per-claim analysis and using an OR rule dramatically improves recall.
    TRIZ P1 (Segmentation) applied to claim detection.
    """
    env = dict(os.environ)
    env.setdefault("RUST_LOG", "error")
    data = run_json_command(
        [str(CLI_BIN), "claims", "--format", "json"],
        input_text=text,
        timeout=30,
        env=env,
    )
    risky = data.get("risky_count", 0)
    claims = data.get("claims", [])
    any_illusion = any(c.get("illusion_issues") for c in claims)
    any_contradiction = any(c.get("antinomy_issues") for c in claims)
    any_paralogism = any(c.get("paralogism_issues") for c in claims)
    overall_risk = data.get("overall_risk", "Safe")
    return {
        "risk": overall_risk,
        "has_illusions": any_illusion,
        "has_contradictions": any_contradiction,
        "has_paralogisms": any_paralogism,
        "risky_claims": risky,
        "error": None,
    }
