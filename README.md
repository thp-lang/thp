# THP

[![CI](https://github.com/thp-lang/thp/actions/workflows/ci.yml/badge.svg)](https://github.com/thp-lang/thp/actions/workflows/ci.yml)
[![Documentation](https://github.com/thp-lang/thp/actions/workflows/docs.yml/badge.svg)](https://github.com/thp-lang/thp/actions/workflows/docs.yml)

THP is an experimental, statically typed, PHP-shaped language for greenfield
programs. It has a standalone compiler, verified bytecode VM, persistent
OPcache, and baseline Cranelift JIT.

THP v0.2.0 is for command-line experiments. It is not production-ready, is not a
PHP-compatible replacement, does not run through the PHP engine, and is not yet
a web backend. Familiar syntax never implies compatibility where THP has not
defined behavior.

## Quick start

Download the archive for your platform from
[Releases](https://github.com/thp-lang/thp/releases), extract it, and
place its `bin` directory on `PATH`. Each archive includes installation notes,
official-binary terms, the Apache source notice, and third-party notices. The
release publishes checksums and a Sigstore signature bundle alongside the
archives.

Create `hello.thp`:

```thp
<?thp

$name: string = "world";
echo "Hello, " . $name . "!\n";
```

Then type-check and run it:

```sh
thp --version
thp check hello.thp
thp run hello.thp
```

To build from source, install Rust 1.88 or newer and run:

```sh
git clone https://github.com/thp-lang/thp.git
cd thp
cargo build --release -p thp-cli
target/release/thp run examples/hello.thp
```

The tested [`examples/project`](examples/project) program shows namespaces,
typed interfaces and classes, collections, project autoloading, and structured
exception handling:

```sh
target/release/thp run --project=examples/project main.thp
```

## What is implemented

THP currently executes an end-to-end compiler/runtime slice:

```text
source → tokens and AST → typed HIR → MIR → verified bytecode → VM
                                                               ↘ JIT
```

The [implementation status](doc/content/guides/implementation-status.md) is the
single detailed authority for supported, partial, and pending behavior. The
documentation separately labels implementation availability and API stability;
pages describing proposals are not promises that the feature executes.

## Command-line interface

```text
thp check [--project=DIR] [--metrics=off|human|json] FILE
thp inspect [--project=DIR] [--emit=tokens|ast|interfaces|module-graph|hir|mir|bytecode] [--metrics=...] FILE
thp run [--project=DIR] [--engine=auto|vm|jit] [--opcache=off|PATH] [--max-instructions=N] [--metrics=...] FILE
thp cache-warm --opcache=PATH [--project=DIR] FILE
thp run --frozen --opcache=PATH [--project=DIR] FILE
thp cache-prune --opcache=PATH [--max-bytes=N] [--metrics=...]
thp --version
```

- `check` compiles and type-checks without executing.
- `inspect` displays compiler and project intermediate forms.
- `run` uses the reference VM or supported JIT subset.
- `cache-warm` publishes a project's interfaces, module objects, linked
  program, and manifest.
- `run --frozen` executes a verified warmed project without scanning source
  directories.
- `cache-prune` constrains a persistent bytecode cache.

`--engine=auto` uses the JIT only when it can preserve VM semantics and falls
back to the VM otherwise. See `thp --help` for the canonical interface.

## Performance measurements

The opt-in synthetic static-module comparison and its reproducible warmup and
measurement methodology are documented in
[`benchmarks/modules/README.md`](benchmarks/modules/README.md). Published results
must include the environment, exact commands, and raw samples; THP does not make
a general performance claim from a single local run.

## Documentation and testing

Language proposals live in `doc/content/language/`; standard-library proposals
live in `doc/content/std/`. The active documentation compiler is `doc/`; no
generated documentation is tracked.

Run the Rust and native specification gates from the repository root:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p thp-test -- tests/phpt/language tests/phpt/resources
```

From `doc/`, run:

```sh
pnpm install --frozen-lockfile
pnpm format:check
pnpm lint
pnpm check
pnpm build
pnpm test:browser
```

## Project

- [Roadmap](ROADMAP.md)
- [Changelog](CHANGELOG.md)
- [Support](SUPPORT.md)
- [Security policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)
- [Compiler and runtime architecture](ARCHITECTURE.md)

External code contributions remain paused until lawyer-reviewed contribution
terms are published. Bug reports, design feedback, and discussions are welcome.

## Licensing

Repository source, documentation, examples, and THP-native tests are licensed
under [Apache-2.0](LICENSE), with attribution in [NOTICE](NOTICE). Specifically
identified official binary releases are additionally offered under the
[MIT License](LICENSE-BINARY). The MIT binary grant does not relicense source or
third-party builds; see [LICENSING.md](LICENSING.md) for the precise boundary.
