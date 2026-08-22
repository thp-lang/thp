---
kind: guide
id: guide.bytecodeInterpreter
title: Bytecode interpreter
summary: Follow verified instructions through VM frames, dispatch, output, failures, and cleanup.
nav:
  section: internals
  order: 100
status: experimental
availability: implemented
notice: >-
  The reference VM executes the implemented language core on one request
  thread.
---

`thp-vm` executes verified bytecode. It receives a program, runtime limits, and
an execution context containing request input, filesystem context, and a host
output sink. It never reads THP source or reconstructs an AST.

```thp
<?thp
function greet(string $name): string {
    return "Hello, " . $name;
}

echo greet("Ada");
```

The bytecode interpreter executes the entry frame, loads `"Ada"` into a
register, pushes a frame for `greet`, returns its concatenated string, and sends
the result to the output sink:

```console
$ thp run greet.thp
Hello, Ada
```

## Frames and dispatch

Each function call owns typed local slots, general value registers, the current
block and instruction position, and source information used for stack frames.
The dispatch loop executes one instruction at a time and then follows the
block's terminator to a return, branch, or next block.

Instructions cover constants, local access, checked scalar operations,
collections, calls, object properties and methods, type tests, output, throws,
and cleanup primitives. Built-ins use the same typed bytecode call boundary as
compiled functions. Virtual and interface calls select a target from verified
class dispatch tables.

## Failures and limits

Checked integer overflow, invalid runtime data, stream failures, uncaught
throwables, and exhausted limits become structured failures with source spans
and THP stack frames. Catch regions and `finally` transfers are already encoded
in bytecode; the VM performs the required unwinding and cleanup.

Request controls include instruction, execution-time, managed-heap,
request-input, logical-stack, and open-handle limits. Output is streamed
synchronously to the host. A captured compatibility wrapper is available for
hosts and tests that need output and counters retained after failure.

## Design choices compared with PHP

The Zend VM executes operations over dynamically tagged values, so an opcode
may select conversions or behavior from the runtime operand types. THP's VM
starts from verified typed descriptors and register representations. It can use
specialized instructions for native vectors, maps, direct calls, and nominal
dispatch, while checked operations report the source span retained in
bytecode.

THP also makes execution budgets part of each VM request: instruction count,
elapsed time, managed heap, input bytes, logical stack, and open handles arrive
together in `Limits`. PHP exposes important controls such as memory and
execution time through runtime configuration, but does not model this same set
as one embedding request contract. THP's explicit budgets make separate hosts
and prepared projects easier to isolate, at the cost of checking more limits in
the execution loop.

The CLI may select [OPcache or the JIT](thp:guide.opcacheJit) around this path;
request values themselves are described in [runtime design](thp:guide.runtimeDesign).
