---
name: test-first-regression-fix
description: Find and repair bugs or regressions in THP changes with a test-first workflow. Use when asked to inspect a diff for correctness and fix confirmed defects, especially compiler, bytecode, VM, runtime, or language-semantics regressions. For review-only requests, report findings without modifying files.
---

# Test-First Regression Fix

Find concrete defects, prove them with the smallest regression test, then fix
each root cause. Do not change code merely because it looks unusual.

## Establish scope

1. Read every applicable `AGENTS.md` before editing.
2. Inspect `git status --short`, then review the combined `git diff HEAD` so
   staged and unstaged changes are both included.
3. Trace each changed behavior through its callers and compiler/runtime phases
   with `rg`. Preserve unrelated user changes and existing staging boundaries.
4. Compare observable behavior with the contracts in `doc/content/language/`,
   `doc/content/std/`, and `implementation-status.md`. Do not infer PHP
   compatibility where THP has no contract.

If the user requested only a review or diagnosis, stop after reporting
evidence-backed findings. Tests and fixes require the user to have requested
changes or explicitly approved follow-up implementation.

## Reproduce before fixing

For every credible defect:

1. Write the smallest test that demonstrates the intended behavior.
2. Prefer a numbered kebab-case PHPT fixture under `tests/phpt/` for observable
   syntax, diagnostics, execution, cleanup, or output. Use a Rust unit test for
   internal invariants that PHPT cannot reach, such as forged bytecode or codec
   validation.
3. Keep related cases in one fixture when they share a root cause; split cases
   when isolation makes the failure materially clearer.
4. Run the new test against the unchanged implementation. Confirm that it
   fails for the suspected reason, not because of malformed syntax, an
   incorrect expectation, or an unrelated limit.
5. If the test passes or cannot reproduce the claim, do not manufacture a fix.
   Reassess the finding and report the evidence.

Never weaken an existing expectation or encode the buggy behavior as the new
contract merely to make a test pass. Update hard-coded PHPT discovery counts
when adding fixtures.

## Fix the shared cause

Patch the narrowest shared layer responsible for the failure. Search all
callers before editing so sibling paths receive the same correction. Preserve
phase boundaries: syntax in syntax, types and narrowing in HIR, lowering in
MIR/bytecode, execution in the VM/runtime, and bytecode trust checks in the
verifier.

Keep diagnostics structured and spans accurate. Do not bypass verifier,
ownership, cleanup, ABI, or module-dependency invariants. Add no abstraction or
dependency unless the existing code and standard library cannot express the
fix simply.

After each fix, rerun the focused regression test. For multiple independent
bugs, keep enough isolation to show which fixes have landed.

## Validate and report

Run the repository checks required by the touched scope. Rust changes normally
require:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Documentation changes require the commands in `doc/AGENTS.md`. Finish with
`git diff --check` and inspect `git status --short`; do not stage or commit
unless requested.

Report:

- which tests failed before the fix and passed afterward;
- the root causes and shared locations changed;
- the full validation results;
- any suspected issue that was not reproducible or intentionally left unfixed.
