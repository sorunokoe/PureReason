# PureReason Improvement Plan

> Generated from hands-on exploration and testing of v0.3.1.

## Findings Summary

### What Works Well
- **Arithmetic repair** — deterministic `A op B = C` repair is reliable and fast.
- **Chain-of-thought verification** — `verify_chain` correctly flags arithmetic errors and accumulates context.
- **MCQ picker** — tie detection with `AmbiguousAnswerError` is well-designed.
- **Guard API** — `ReasoningGuard` provides a clean, simple entry point.
- **Degradation tracking** — `_ReputationTracker` is a practical production feature.

### Issues Found

| # | Issue | Severity | Status |
|---|-------|----------|--------|
| 1 | Word-number extraction fails without `word2number` installed — tests expect it but the package is optional | Medium | Documented |
| 2 | Examples were generic and untested — no way for consumers to validate setup | High | **Fixed** |
| 3 | `verify_chain` falls back to ECS=50 when Rust binary is unavailable — no clear indication to the user | Medium | Documented |
| 4 | `_ecs_score` in `guard.py` silently returns 75.0 on any exception — masks real failures | Medium | Documented |
| 5 | Examples README referenced `verify_chain(llm_output)` with wrong signature (missing `steps` parameter) | High | **Fixed** |
| 6 | No test coverage for example use cases | High | **Fixed** |
| 7 | `solve_arithmetic` relies on spaCy NLP model but error message is unclear | Low | Documented |

---

## Improvement Plan

### Phase 1: Examples & Documentation (completed)

- [x] Create 6 focused, tested example files covering every Python use case
- [x] Add `tests/test_examples.py` with 36 tests validating all examples
- [x] Rewrite `examples/README.md` with per-use-case documentation
- [x] Update `README.md` with accurate code samples and API references
- [x] Document expected inputs, outputs, and edge cases

### Phase 2: Robustness (recommended next)

- [ ] **Graceful fallback messaging** — When the Rust binary is unavailable,
  `_ecs_score` should log a clear warning (not silently return 75.0).
  Suggested: use `warnings.warn()` on first fallback.

- [ ] **Optional dependency handling** — `_extract_numbers` silently skips
  word-form numbers when `word2number` is not installed.  Add a one-time
  warning so users know they're missing functionality.

- [ ] **Consolidate install instructions** — The `pyproject.toml` optional groups
  (`[nlp]`, `[logic]`, `[semantic]`, `[rest]`) should be documented in a
  single "Installation" section in the README so users know what each extra
  provides.

### Phase 3: Test Coverage

- [ ] **Integration tests with Rust binary** — Add a CI job that builds the
  Rust binary and runs tests without mocking `_core._run`.

- [ ] **Benchmark regression tests** — Add a small smoke-test subset of
  the HaluEval/TruthfulQA benchmarks that runs in CI to catch ECS score
  regressions.

- [ ] **Property-based testing** — Use `hypothesis` for arithmetic repair
  to verify `_repair_arithmetic_in_step` handles edge cases like very large
  numbers, unicode operators, and chained expressions.

### Phase 4: API Ergonomics

- [ ] **Typed return objects everywhere** — `pick_best_answer` returns
  `tuple[int, EpistemicChainReport]` which is not self-documenting.
  Consider a `MCQResult` dataclass.

- [ ] **Batch verification API** — `ReasoningGuard.verify_batch(texts)` to
  verify multiple texts in a single call (parallel processing).

- [ ] **Structured error types** — Replace generic `Exception` catches with
  specific error types (`BinaryNotFoundError`, `ParseError`, etc.).

### Phase 5: Performance

- [ ] **Lazy NLP model loading** — spaCy model is loaded on first call to
  `_detect_operation`.  Add explicit `init()` method for applications that
  want to control startup latency.

- [ ] **Caching** — `_ecs_for_text` could cache results for repeated texts
  (LRU cache with configurable size).

---

## Priority Matrix

| Priority | Effort | Items |
|----------|--------|-------|
| **High / Low effort** | Phase 1 (done), Phase 2 fallback warnings | 
| **High / Medium effort** | Phase 3 integration tests |
| **Medium / Low effort** | Phase 4 typed returns |
| **Medium / High effort** | Phase 5 performance |

## Recommendation

Start with **Phase 2** (robustness) — it's low-effort and directly improves the
developer experience for new consumers.  Then move to **Phase 3** (test coverage)
to prevent regressions as the project grows.
