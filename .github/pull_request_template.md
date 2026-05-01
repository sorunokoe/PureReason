## Summary

<!-- One-line description of the change -->

## Type

- [ ] Bug fix
- [ ] New feature / enhancement
- [ ] Detection improvement (prior, oracle, heuristic)
- [ ] Refactor / cleanup
- [ ] Documentation
- [ ] CI / tooling

## Description

<!-- What does this PR do and why? -->

## Testing

- [ ] Python unit tests pass: `python3 -m unittest discover -s tests -p "test_*.py" -v`
- [ ] Ruff clean: `ruff check . && ruff format --check .`
- [ ] Rust tests pass (if Rust code changed): `cargo test`
- [ ] Benchmark regression checked (if detection logic changed):
  `python3 benchmarks/run_benchmarks.py`

## Benchmark impact (detection PRs only)

<!-- If you changed detection logic, fill in before/after F1 scores.
     Run: python3 benchmarks/run_benchmarks.py
     Leave blank for non-detection PRs. -->

| Benchmark | Before | After | Delta |
|---|---|---|---|
| HaluEval QA | | | |
| TruthfulQA | | | |

## Checklist

- [ ] No LLM, API key, or external model dependency introduced
- [ ] All Python files ≤ 400 lines (exceptions need justification)
- [ ] Public API documented (docstrings / `///` for Rust)
- [ ] No commented-out code left in
- [ ] `.github/CONTRIBUTING.md` consulted for code style guidelines

## Related issues

<!-- Closes #... -->
