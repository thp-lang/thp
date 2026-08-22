---
kind: guide
id: guide.internalsOverview
title: Overview
summary: Follow THP source through the standalone compiler, bytecode interpreter, and request runtime.
nav:
  section: internals
  order: 10
status: experimental
availability: implemented
notice: >-
  These pages describe the current implementation. Internal formats are
  debugging interfaces and may change with the compiler.
---

THP is implemented as a standalone language. It does not translate a program
to PHP and it does not ask the PHP engine to execute it. Source crosses a series
of one-way boundaries before the runtime sees it:

```text
source files
  → tokens
  → spanned AST units
  → module interfaces, export index, and dependency graph
  → resolved and typed HIR
  → control-flow MIR
  → linked and verified bytecode
  → bytecode VM
       ↘ supported functions may execute through the Cranelift JIT
```

Each representation removes choices from the next phase. The parser decides
syntax, HIR decides names and types, MIR makes control flow and cleanup
explicit, and bytecode is the only program representation accepted by the VM.
No later phase reparses source or supplies missing language semantics.

## Inspecting the pipeline

The `thp inspect` command exposes the compiler's debugging representations:

```console
thp inspect --emit=tokens hello.thp
thp inspect --emit=ast hello.thp
thp inspect --emit=hir hello.thp
thp inspect --emit=mir hello.thp
thp inspect --emit=bytecode hello.thp
```

With `--project=DIR`, it can also emit `interfaces` and `module-graph` for a
configured project. A phase is omitted when an earlier diagnostic prevents it
from being built. These dumps are intended for analysis, not as stable file
formats.

`--metrics=human` and `--metrics=json` measure source loading, discovery,
lexing, parsing, interface extraction, HIR, MIR, linking or bytecode lowering,
verification, cache work, VM execution, and JIT compilation. The JSON metrics
schema is versioned; the textual IR dumps are not.

## Phase ownership

Compiler responsibilities are split across one-way Rust crates:

| Phase                 | Owner               | Result                                             |
| --------------------- | ------------------- | -------------------------------------------------- |
| Source syntax         | `thp-syntax`        | Tokens, spanned AST, syntax diagnostics            |
| Static projects       | `thp-modules`       | Interfaces, exports, graph, resolved names         |
| Semantic analysis     | `thp-hir`           | Typed functions, classes, locals, and expressions  |
| Control-flow lowering | `thp-mir`           | Basic blocks, registers, handlers, terminators     |
| Executable format     | `thp-bytecode`      | Versioned register bytecode and verification       |
| Execution             | `thp-vm`, `thp-jit` | Interpreted or supported native execution          |
| Request values        | `thp-runtime`       | Values, heap cells, streams, failures, and cleanup |

`thp-compiler` orchestrates these phases and records metrics. Presentation
layers render diagnostics and output; compiler phases return structured data
instead of printing or exiting.

## Design choices compared with PHP

THP keeps PHP-shaped syntax where it remains useful, but chooses different
constraints for predictability and static analysis:

| Area                    | THP choice                                                                              | PHP choice                                                                       |
| ----------------------- | --------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| Program composition     | Discover and link a configured module graph before execution                            | Allow files and declarations to be loaded as execution proceeds                  |
| Type discipline         | Type every implemented expression and reject incompatible operations during compilation | Keep values dynamic and enforce declared types at selected boundaries            |
| Conditions and equality | Require `bool` conditions and compatible operand types                                  | Define truthiness and broad comparison/coercion rules                            |
| Collections             | Separate generic `vector<T>` and `map<K, V>` values                                     | Use one general-purpose ordered `array` type                                     |
| Executable input        | Verify a versioned, linked bytecode program before execution                            | Compile scripts to engine-internal op arrays, including files loaded at runtime  |
| Host boundary           | Pass request input, output, filesystem context, and limits explicitly                   | Expose request state primarily through the SAPI, configuration, and superglobals |

These choices trade some of PHP's runtime composition and coercive convenience
for deterministic builds, earlier diagnostics, typed IR, and request behavior
that embedding hosts can configure without process-global state. See the
[implementation status](thp:guide.implementationStatus) for the exact
executable feature boundary.

## Continue through the stages

Start with [source loading and project discovery](thp:guide.sourceLoading), or
jump to the [bytecode interpreter](thp:guide.bytecodeInterpreter) and
[runtime design](thp:guide.runtimeDesign) when investigating execution.
