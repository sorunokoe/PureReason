#!/usr/bin/env bash
# Release readiness gate for PureReason.
# Usage: bash scripts/release-check.sh
# Exit 0 = ready, exit 1 = blockers found.
set -euo pipefail

PASS=0
FAIL=0

ok()   { echo "  ✅  $*"; PASS=$((PASS + 1)); }
fail() { echo "  ❌  $*"; FAIL=$((FAIL + 1)); }
info() { echo "  ℹ️   $*"; }

echo ""
echo "═══════════════════════════════════════"
echo "  PureReason Release Readiness Check"
echo "═══════════════════════════════════════"
echo ""

# ── 1. Clippy gate ────────────────────────────────────────────────────────────
echo "▶ Clippy (all targets, deny warnings)…"
if cargo clippy --workspace --all-targets -- -D warnings 2>&1 | grep -q "^error"; then
  fail "Clippy errors found — run: cargo clippy --workspace --all-targets -- -D warnings"
else
  ok "Clippy clean"
fi

# ── 2. Tests ──────────────────────────────────────────────────────────────────
echo "▶ Tests…"
if cargo test --workspace --quiet 2>&1 | grep -q "^FAILED\|^error"; then
  fail "Test failures — run: cargo test --workspace"
else
  ok "All workspace tests pass"
fi

# ── 3. Dirty working tree ─────────────────────────────────────────────────────
echo "▶ Working tree…"
# Exclude target/ (build artifacts) from dirty-tree check
DIRTY=$(git diff --name-only | grep -v '^target/' | wc -l | tr -d ' ' || true)
DIRTY_CACHED=$(git diff --cached --name-only | grep -v '^target/' | wc -l | tr -d ' ' || true)
if [ "$DIRTY" -eq 0 ] && [ "$DIRTY_CACHED" -eq 0 ]; then
  ok "Working tree clean (excluding build artifacts in target/)"
else
  fail "Uncommitted source changes — commit or stash before release"
fi

# ── 4. benchmark/results not tracked ─────────────────────────────────────────
echo "▶ Benchmark artifacts in git…"
# Only block if NEW benchmark artifacts are staged or untracked (not already-committed reference data)
STAGED_ARTIFACTS=$(git diff --cached --name-only -- 'benchmarks/results/*' 'benchmarks/downloads/*' 2>/dev/null | wc -l | tr -d ' ')
UNTRACKED_ARTIFACTS=$(git ls-files --others --exclude-standard -- 'benchmarks/results/' 'benchmarks/downloads/' 2>/dev/null | wc -l | tr -d ' ')
if [ "$STAGED_ARTIFACTS" -gt 0 ] || [ "$UNTRACKED_ARTIFACTS" -gt 0 ]; then
  fail "New benchmark artifacts staged or untracked — add to .gitignore (benchmarks/results/, benchmarks/downloads/)"
else
  ok "No new benchmark artifacts pending commit"
fi

# ── 5. docs/BENCHMARK.md has 95% CIs ─────────────────────────────────────────
echo "▶ Confidence intervals in docs/BENCHMARK.md…"
if grep -q "CI\|ci_95\|±" docs/BENCHMARK.md; then
  ok "Confidence intervals present in docs/BENCHMARK.md"
else
  fail "docs/BENCHMARK.md missing confidence intervals — run benchmarks with n≥200 + Wilson CIs"
fi

# ── 6. protocol.yaml present ─────────────────────────────────────────────────
echo "▶ Evaluation protocol…"
if [ -f benchmarks/protocol.yaml ]; then
  ok "benchmarks/protocol.yaml exists"
else
  fail "benchmarks/protocol.yaml missing — scientific reproducibility requires it"
fi

# ── 7. docs/capabilities.json present ────────────────────────────────────────
echo "▶ Capability manifest…"
if [ -f docs/capabilities.json ]; then
  ok "docs/capabilities.json exists"
else
  fail "docs/capabilities.json missing — README/CLI claims must be driven from it"
fi

# ── 8. No plain-text TODO/FIXME in crate source ──────────────────────────────
echo "▶ FIXME / TODO in crate source…"
COUNT=$(grep -rn "FIXME\|TODO:" crates/ --include="*.rs" | wc -l | tr -d ' ' || true)
if [ "$COUNT" -gt 0 ]; then
  info "Found $COUNT FIXME/TODO in crates/ (not blocking, but review before release)"
else
  ok "No FIXME/TODO markers in crate source"
fi

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
echo "═══════════════════════════════════════"
printf  "  Passed: %d  Failed: %d\n" "$PASS" "$FAIL"
echo "═══════════════════════════════════════"
echo ""

if [ "$FAIL" -gt 0 ]; then
  echo "🔴  Release blocked — fix the failures above."
  exit 1
else
  echo "🟢  Release ready."
  exit 0
fi
