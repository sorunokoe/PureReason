# Reproducibility protocol

This document tracks how benchmark numbers in `README.md` are produced and
how a third party can verify them on a clean machine.

## Source of truth

Every headline F1 quoted in the README must correspond to a committed JSON
summary under `benchmarks/results/`. Each summary carries:

- `version` — detector variant (e.g. `downloaded_heuristic`)
- `holdout` — `true` if the sample draw was never used for threshold tuning
- `sample_size_per_class` — `n` per class (balanced)
- Per-benchmark metrics (`precision`, `recall`, `f1`, `accuracy`, `ci_95`)

## Hashes

The SHA256 (and BLAKE3, when the cert toolchain is available) of each
summary JSON is committed here so post-hoc edits to the summary show up as
a diff on this file:

| File | SHA256 | Produced |
|---|---|---|
| `benchmarks/results/SUMMARY_full.json` | `719a0bdd126ae6f4754a37928ab645d8a79c8c24c71dc07559f5f786facf9987` | 2026-04-28 |
| `benchmarks/results/SUMMARY_holdout.json` | `bd1cec058e22a7f1f718c7c688aa55e51ca2dbf0873c2bdb0944c861bff102c6` | 2026-04-28 |

When a benchmark summary changes, the CI job must regenerate this table and
fail if a hash drifts without a matching commit to this file.

## Seeds

All randomness is seeded:

- Benchmark sampling uses `random.Random(seed)` with `--seed 42` as default.
- Holdout mode (`--holdout`) evaluates on `rows[n : 2n]` — disjoint from the
  `rows[0 : n]` used for threshold calibration.

## Replication

```bash
# 1. Clone and build
git clone https://github.com/sorunokoe/PureReason
cd PureReason
cargo build --release -p pure-reason-cli

# 2. Fetch benchmark data
python3 benchmarks/download_benchmarks.py --benchmarks all

# 3. Audit for leakage (must pass)
python3 benchmarks/benchmark_leak_audit.py --verbose

# 4. Produce the standard summary
python3 benchmarks/run_downloaded_benchmarks.py \
    --benchmarks all --n 200 --seed 42 \
    --output benchmarks/results/SUMMARY_downloaded_heuristic.json

# 5. Produce the holdout summary
python3 benchmarks/run_downloaded_benchmarks.py \
    --benchmarks all --n 200 --seed 42 --holdout \
    --output benchmarks/results/SUMMARY_holdout.json

# 6. Verify hashes match this document
shasum -a 256 benchmarks/results/SUMMARY_*.json
```

The two summaries must be published **together**. If the holdout F1 is
materially lower than the full-split F1, the headline number is the
**holdout** F1.

## SOTA comparisons

The README must not compare PureReason's heuristic F1 to an external
model's F1 unless the baseline was run on the same sample by this repo.
In practice, that means:

- Run the external baseline through a reproducible in-repo harness on the same
  `(benchmark, n, seed)` triple;
- Publish *both* F1 values side by side in the README, not a cherry-picked
  external number.

## Why this exists

TRIZ-42 §6 / NE-7: the previous README cited an "S53 holdout" F1 table for
which **no result file with `holdout: true` had ever been committed**. This
protocol fixes that by making the JSON summary, its hash, and the command
that produced it the primary artifacts, with the README only summarising
them.
