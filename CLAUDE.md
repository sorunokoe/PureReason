# CLAUDE.md

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.

---

## 5. Spec Driven Development Workflow

**This workflow is MANDATORY for every feature, refactor, or non-trivial bugfix. No exceptions.**

The core principle: write the spec (type + test) *before* the implementation. The compiler is your first test runner.

```
1. CLARIFY      Ask questions until the scope is unambiguous. Never assume.
2. SPEC         Write the feature spec: types + behavior + acceptance criteria (Given-When-Then).
3. PLAN         Create an implementation plan. Present it for human approval before writing code.
4. TYPE DESIGN  Define or update types. Make invalid states unrepresentable at compile time.
5. TEST FIRST   Write failing Swift Testing specs that encode the acceptance criteria.
6. IMPLEMENT    Write the minimum code needed to pass all tests. Nothing speculative.
7. VALIDATE     Build + run tests + type-safety review (no raw strings, no bool flag pairs).
8. DOCUMENT     Update DocC articles and doc comments.
```

**You must not skip steps 1–3.** If you find yourself writing implementation code before tests exist, you have skipped the process.

For the detailed playbook, templates, and checklist → invoke the `spec-driven-development` skill.

### Type-Driven Design (step 4) — Quick Reference

- Replace `String` for domain concepts with `enum` → compiler enforces exhaustive handling
- Replace `Bool` flag pairs (`isLoading + data?`) with a state `enum`
- Replace mutually exclusive `Optional` fields with an `enum`
- New `enum` conforming to `String, RawRepresentable` preserves Codable/SwiftData compatibility

See `swift-type-driven-design` skill for full patterns and code examples.

### SDD Exit Criteria (must satisfy all before marking done)

- [ ] Scope clarified with explicit questions before coding started
- [ ] Feature spec written (types + Given-When-Then acceptance criteria)
- [ ] Implementation plan approved before coding
- [ ] All new domain concepts are types, not raw `String` or `Int`
- [ ] No `Bool` flag pairs — state modeled as enum
- [ ] Tests written *before or alongside* implementation (not after)
- [ ] All tests pass (`swift test` in the relevant package)
- [ ] DocC and doc comments updated
