---
kind: guide
id: guide.opcacheJit
title: OPcache and JIT
summary: Learn how THP reuses verified artifacts and selects its restricted Cranelift execution tier.
nav:
  section: internals
  order: 110
status: experimental
availability: partial
notice: >-
  OPcache is implemented; the current JIT intentionally accepts only a
  restricted safe subset and otherwise uses the VM.
---

OPcache avoids repeating compilation, while the JIT provides a second execution
engine for bytecode it can reproduce exactly. Neither changes THP language
semantics.

```console
thp run --opcache=.thp-cache app.thp
thp run --engine=auto --metrics=human scalar.thp
```

## Content-addressed artifacts

A single-source cache key covers source bytes, compiler and bytecode identity,
effective configuration, bytecode schema, operating system, and architecture.
Entries are immutable and published atomically. A hit is fully decoded and
verified before use; a missing, corrupt, or incompatible normal entry compiles
again.

Project cache warming uses separate artifacts:

| Extension | Contents                                      |
| --------- | --------------------------------------------- |
| `.thpi`   | Extracted module interface                    |
| `.thpo`   | Compiled module object                        |
| `.thpbc`  | Linked bytecode program                       |
| `.thpm`   | Frozen manifest referencing the completed set |

The manifest is published last. Frozen execution validates project and format
identity, configuration, entry selection, referenced artifacts, and the linked
program without scanning mapped source directories.

```console
thp cache-warm --project=. --opcache=.thp-cache main.thp
thp run --project=. --frozen --opcache=.thp-cache main.thp
```

## Cranelift selection

`--engine=vm` always uses the interpreter. `--engine=jit` requests the
Cranelift tier and reports an unsupported-program error when the bytecode lies
outside its current subset. `--engine=auto` checks the whole program first and
uses the VM when native execution cannot preserve identical behavior.

The current tier handles straight-line scalar functions, locals, direct calls,
comparisons, and boolean or null tests. Heap values, general control-flow
graphs, checked arithmetic that can fail, output, objects, method dispatch,
exceptions, cleanup instructions, and instruction limits remain on the VM
path. JIT compilation has its own metrics stage.

## Design choices compared with PHP

PHP OPcache is centered on reusing compiled scripts and commonly validates
them from file identity and timestamps. THP keys immutable artifacts from their
content plus compiler, configuration, format, OS, and architecture identity.
It separately caches module interfaces, module objects, and the final linked
program, so a project build can reason about dependencies before publishing a
manifest. The result favors reproducibility and frozen deployment over
timestamp-driven development convenience.

PHP JIT can compile selected functions or traces while the rest of a request
continues in the Zend VM. THP's current `auto` policy is deliberately more
conservative: it checks the complete program and selects Cranelift only when
the supported subset can preserve every operation's behavior; otherwise the
whole request uses the VM. This avoids mixed-tier state and deoptimization in
the first tier, but limits how often native execution is selected.

Both execution paths use the same [runtime values and request model](thp:guide.runtimeDesign).
