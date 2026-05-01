# TRIZ-42 — Integrity and evolution report (2026-04-22)

Source: project audit, 2026-04-22. This document tracks the undesirable
effects (NEs) found and the remediation status of each.

## Undesirable effects

| # | NE | Source | Status |
|---|---|---|---|
| NE-1 | Test-set label leakage on TruthfulQA (`_load_truthfulqa_myths` read `Incorrect Answers` column into verdict DB) | `benchmarks/detectors/truthfulqa.py` | **Fixed** (`_load_truthfulqa_myths` deleted; verdict routed through `universal_verdict`) |
| NE-2 | 107-entry hand-crafted prior atlas targeting canonical TruthfulQA / FELM items | `crates/pure-reason-core/src/world_priors.rs` | Not removed yet; see §"External-corpus priors" below |
| NE-3 | `mine_failures.py` generated Rust snippets for `world_priors.rs` from test-set failures (fitting loop) | `scripts/mine_failures.py` | **Fixed** (output is now a markdown diagnostic taxonomy; code generation removed) |
| NE-4 | Per-benchmark verdict functions with dataset-specific thresholds | `benchmarks/detectors/verdicts.py` | **Fixed** (collapsed to `universal_verdict`; per-bench names kept as deprecated aliases) |
| NE-5 | README "Zero LLM · Zero API" contradicted the repo's former direct-provider path | `README.md` | **Fixed** (repo now presents only the local agent-facing verifier path) |
| NE-6 | SOTA comparison vs Lynx-70B / GPT-4 used heuristic numbers against LLM baselines | `README.md` | **Fixed** (SOTA comparison removed until matched-run is published) |
| NE-7 | "S53 holdout protocol" referenced in docs but no `holdout:true` artifact committed | `benchmarks/results/` | Protocol documented in `docs/REPRODUCIBILITY.md`; holdout JSON still needs to be produced by CI |
| NE-8 | CI regression gate defanged 2 days before audit (`--fail-on-error` removed from workflow) | `.github/workflows/epistemic-regression.yml`, `pure-reason-bench/src/main.rs` | **Fixed** (gate restored; default is `true`; env var unmaskable) |
| NE-9 | Non-determinism in `unity.rs` (HashMap iteration) | `crates/pure-reason-core/src/unity.rs:200` | **Fixed** (BTreeMap + explicit key tiebreak) |
| NE-10 | MCQ picker returns first choice on ECS ties (silent guess) | `pureason/reasoning/mcq.py` | **Fixed** (ambiguity now flagged; `strict=True` raises `AmbiguousAnswerError`) |
| NE-11 | Arithmetic tolerance fixed at 0.01 — missed small-magnitude errors | `pureason/reasoning/chain.py`, `repair.py`, `felm_oracles.py` | **Fixed** (floor lowered to 1e-6) |
| NE-12 | Outbound webhook dispatch contradicted "offline" claim | `crates/pure-reason-api/src/main.rs` | In progress — will move behind `--features webhooks` |

## External-corpus priors (follow-up for NE-2)

`crates/pure-reason-core/src/world_priors.rs` currently contains 107
entries with IDs directly mirroring canonical TruthfulQA items
(`seasons_sun_distance`, `chili_seeds_spiciest`, `watermelon_seeds_stomach`,
…). The structural fix is:

1. Replace the static array with a table loaded from `data/misconceptions_corpus_v1.jsonl`.
2. Populate that file from Wikipedia's "List of common misconceptions"
   (CC-BY-SA), committed with source URL and SHA256 per entry.
3. Run `benchmarks/benchmark_leak_audit.py` in CI. Fail on any hit.
4. Report the post-migration F1 alongside the pre-migration F1 in
   `docs/REPRODUCIBILITY.md`.

Until that migration lands, the leak-audit script is the containment
control: it parses the Rust atlas and flags any signal string that appears
in a benchmark's test data.

## Evolution target

| Stage | Description | Status |
|---|---|---|
| Mono | Heuristic detector with dataset-tuned rules | — |
| **Bi** | Heuristic detector with external-corpus priors and explicit reviewable reporting | Bi partially landed (universal verdict + webhook feature flag); corpus migration pending |
| Poly | Ensemble of heuristic / symbolic / semantic detectors with calibrated uncertainty | Not started |
| Field | Self-auditing: signed summaries, automatic drift alerts, third-party replication badge | Leak audit exists; signing next |

## Rerun protocol after landing these fixes

```bash
cargo fmt
cargo check --all-targets
cargo test -p pure-reason-core -p pure-reason-bench
python3 -m pytest tests/
python3 benchmarks/benchmark_leak_audit.py --verbose
python3 benchmarks/run_downloaded_benchmarks.py --benchmarks all --n 200 --seed 42
python3 benchmarks/run_downloaded_benchmarks.py --benchmarks all --n 200 --seed 42 --holdout \
    --output benchmarks/results/SUMMARY_holdout.json
```

Both summaries and their SHA256s get committed together with an updated
`docs/REPRODUCIBILITY.md`.
