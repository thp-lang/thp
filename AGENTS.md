# Repository Guidelines

## Project Scope

THP is an experimental, typed, PHP-shaped language intended for a standalone
compiler and runtime. An experimental implementation now covers a vertical
compiler, bytecode interpreter, object slice, OPcache, and baseline JIT. Current language
contracts live in `doc/content/language/`; standard-library contracts live in
`doc/content/std/`. Treat these as proposals unless a page explicitly says
otherwise. Do not claim compatibility with PHP where THP has not defined it.

`tests/phpt/resources/` contains behavioral specification fixtures for the
proposed resource and stream model. `doc/` contains the active documentation
compiler. Read `doc/AGENTS.md` before changing that application. Historical
documentation prototypes are available only through Git history and are not
language authorities.

`crates/` contains the Rust 2024 implementation workspace. Compiler phases have
separate crates for syntax, HIR, MIR, bytecode, runtime values, the VM, JIT,
static module discovery/indexing, OPcache, metrics, orchestration, safe
embedding/SAPI adapters, the versioned C ABI, the CLI, and the reusable
`thp-test` PHPT runner; preserve those one-way
dependencies. Run selected PHPT files with
`cargo run -p thp-test -- [--max-instructions=N] PATH...`. The public C header is
`crates/abi/include/thp.h`; keep it synchronized with its `#[repr(C)]` Rust
definitions and never expose a Rust layout through that boundary. ABI version
1 is the only recognized ABI version until the repository owner explicitly
authorizes a change; do not increment it or accept compatibility version
numbers in the meantime.
`crates/config` owns project TOML loading, target resolution, structured
configuration diagnostics, deterministic lock generation, and the fast lock
parser. Keep extension configuration lazy on the lock-file path: loading core
runtime settings must not require parsing extension-owned TOML. See
`ARCHITECTURE.md` for phase and runtime invariants.
`crates/modules` owns module IDs, source providers, autoload discovery, export
interfaces, dependency edges, deterministic graph order, SCCs, and project
name resolution. It must not depend on HIR, MIR, bytecode, the VM, or platform
presentation layers.
`benchmarks/modules/` is the opt-in PHP/THP static-module comparison suite.
Follow its README, warm every artifact before measurement, and record the
environment and exact commands with results; do not add hardware-sensitive CI
thresholds.

## Language Design Changes

Define observable behavior before implementation. A language proposal should
state:

- accepted and rejected syntax;
- static typing and inference rules;
- runtime semantics, including ownership and cleanup;
- failure behavior and diagnostic locations;
- interactions with existing features and the standard library.

Include small `<?thp` examples and identify unresolved decisions rather than
silently inheriting PHP behavior. Keep terminology consistent across related
pages. Mark unstable contracts `status: experimental`, declare implementation
`availability: implemented | partial | proposed`, and describe what remains
unimplemented in the notice. Keep `implementation-status.md` as the single
detailed feature authority.

## Compiler & Runtime Architecture

When implementation is introduced, keep phases explicit: source loading,
lexing, parsing, name resolution, type checking, lowering, and execution.
Parser nodes should preserve source spans; later phases should return structured
diagnostics instead of printing or exiting. Keep syntax decisions out of the VM
and platform I/O out of the type checker. Add new top-level source directories
and their build commands to this guide in the same change that introduces them.

Unsafe Rust is permitted in the allocation tracker and low-level runtime, ABI,
and JIT primitives when the safety invariant is documented at the unsafe block.
Do not use unsafe code to bypass ordinary ownership or phase-boundary design.

## Testing Semantics

Every implemented feature needs positive, negative, and boundary tests. Prefer
unit tests for compiler phases and end-to-end fixtures for observable output and
diagnostics. PHPT files use numbered kebab-case names and standard `--TEST--`,
`--FILE--`, `--EXPECT--`, or `--EXPECTF--` sections; follow
`tests/phpt/resources/README.md`. They are specifications today and must not be
run through PHP. When a THP runner exists, compile-error expectations must count
as passing tests.

## Documentation Validation

From `doc/`, use `pnpm check`, `pnpm lint`, `pnpm format:check`, and `pnpm build`
for language or API documentation changes. Never edit generated `doc/dist/`.

For Rust changes, use `cargo fmt --all -- --check`, `cargo check --workspace`,
`cargo clippy --workspace --all-targets -- -D warnings`, and
`cargo test --workspace` from the repository root. `Cargo.lock` is tracked
because the workspace produces THP tooling, even when its initial members are
libraries.

## Commits & Pull Requests

Use short imperative Conventional Commit subjects, such as `feat: define union
type narrowing`. Keep design, tests, and implementation synchronized. Pull
requests should summarize semantic decisions, list unresolved questions and
validation commands, link relevant issues, and call out compatibility or
diagnostic changes.
