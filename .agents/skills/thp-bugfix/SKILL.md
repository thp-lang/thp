---
name: thp-bugfix
description: Reproduce and fix THP compiler, runtime, or LSP bugs with a failing regression test first. Use for defect investigation and hardening; not for feature design.
---

# THP bugfix

Trace the affected behavior and every caller of the shared path before editing.

Choose the smallest executable regression test for the public behavior:

- Use a numbered `.phpt` fixture only for compiler/runtime behavior that the THP test runner executes. Run it with `cargo run -p thp-test -- PATH`.
- Use a Rust unit or integration test for compiler diagnostics, source mapping, LSP protocol messages, editor state, or process exit behavior. Do not force LSP behavior into PHPT.

Add the regression test before the fix and run it to demonstrate the failure. Fix the shared root cause, then rerun that test. Keep tests focused on the observed failure and its boundary; do not add a framework or duplicate existing coverage.

For LSP changes, exercise the actual stdio binary when the failure involves framing, lifecycle, exit status, or client-visible requests. Keep protocol traffic on stdout; assertions may inspect stderr only for a failed process.

Run the narrow test first. For Rust changes, finish with `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`. Report the reproduced bug, test that caught it, fix location, and validation.
