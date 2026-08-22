---
kind: guide
id: guide.bytecodeVerification
title: Bytecode verification
summary: Understand the checks that keep malformed or incompatible bytecode out of the execution engines.
nav:
  section: internals
  order: 90
status: experimental
availability: implemented
notice: >-
  Verification is mandatory for generated and cached bytecode before
  execution.
---

Decoding proves that bytes can be represented as bytecode structures.
Verification proves that those structures form a valid THP program. The VM and
JIT accept only a program that has passed both boundaries.

For example, this conceptual instruction is invalid when function `#3` owns
only two registers:

```text
r2 = Binary { op: Add, left: Register(0), right: Register(7) }
```

The verifier rejects the out-of-range operand before an execution engine can
read it.

## Verified invariants

The verifier checks the current bytecode model as a whole:

- entry, function, class, property, method, block, register, and local IDs refer
  to existing descriptors;
- descriptor tables agree about inheritance, dispatch slots, constructors,
  properties, and method signatures;
- instructions receive and produce the required register representations;
- calls have a valid target, argument count, parameter types, and return use;
- block terminators target existing blocks and return values match the function;
- exception-handler ranges and catch targets are structurally valid;
- built-in calls obey their declared operand and result contracts.

Verification errors are converted into structured compiler diagnostics for
fresh compilation. A missing or corrupt cache entry is not partially executed;
normal cached compilation treats it as a miss and builds a verified program
again. Frozen execution fails validation rather than scanning source as a
fallback.

## Why verification remains separate

The compiler normally emits correct bytecode, but persisted artifacts cross a
trust and version boundary. Files can be truncated, corrupted, produced by a
different format version, or supplied by an embedding host. Keeping
verification in `thp-bytecode` means both the VM and JIT share one executable
input contract.

The verifier does not repeat source type inference. It validates the explicit
types, descriptors, and operations already encoded in the program.

## Design choices compared with PHP

PHP keeps op arrays as engine-owned structures and OPcache protects them with
engine, platform, and cache compatibility rules. They are not a typed
distribution boundary intended for arbitrary producers.

THP makes serialized bytecode and host-provided cached programs an explicit
input boundary, so it runs one mandatory verifier after decoding. The verifier
checks the complete descriptor graph, register representations, calls,
handlers, and control-flow targets—not only the file header or schema version.
That adds load-time work, but the VM and JIT can rely on the same invariants and
do not need defensive checks for malformed IDs at every dispatch.

A verified program can enter the [bytecode interpreter](thp:guide.bytecodeInterpreter).
