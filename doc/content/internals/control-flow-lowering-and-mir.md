---
kind: guide
id: guide.controlFlowMir
title: Control-flow lowering and MIR
summary: Learn how typed statements become register operations, basic blocks, handlers, and explicit transfers.
nav:
  section: internals
  order: 70
status: experimental
availability: implemented
notice: >-
  MIR is an internal control-flow representation and its text form may change.
---

HIR still resembles source statements. Middle IR (MIR) removes that structured
surface and makes execution order explicit with registers, basic blocks,
instructions, and terminators.

```thp
<?thp
function label(bool $ready): string {
    if ($ready) {
        return "ready";
    }
    return "waiting";
}
```

The condition becomes a branch terminator targeting two block IDs. Each return
becomes a block terminator, so bytecode generation does not need to understand
the syntax of `if`.

For a straight-line function, current `thp inspect --emit=mir` output looks
like this abbreviated excerpt:

```text
fn greet#1 (locals=1, registers=3) -> string {
  bb0:
    r0 = Constant(String([72, 101, 108, 108, 111, 44, 32]))
    r1 = LoadLocal(LocalId(0))
    r2 = Binary { op: Concatenate, left: Register(0), right: Register(1) }
    -> Return(Some(Register(2)))
}
```

## Lowered behavior

MIR lowers loops, short-circuit boolean operators, null coalescing, native
collection iteration, indexing, calls, object operations, and `match` into
explicit control flow and primitive instructions. An unreachable-block pass
removes blocks that can no longer be entered.

Exception handlers describe protected block ranges and ordered catches.
`finally` and `using` are lowered with explicit cleanup transfers so that
returning, throwing, breaking, or continuing cannot silently skip required
cleanup. Transfer replacement and suppressed cleanup failures are represented
before the VM runs.

Every instruction and terminator retains a source span. Runtime failures can
therefore identify the originating expression even though the VM never reads
the AST.

## Design choices compared with PHP

PHP's compiler lowers its AST to Zend op arrays while preserving operations
that dispatch on dynamic runtime values. THP inserts a typed control-flow IR
between its AST-like HIR and bytecode. MIR gives branches, registers, exception
regions, and exits from `finally` or `using` one explicit representation before
an instruction encoding is selected.

The additional phase costs compiler time and another internal format, but it
centralizes cleanup correctness and makes control-flow transformations
independent of both source syntax and VM encoding. Because MIR instructions
already have typed operands, bytecode generation can be mostly mechanical
rather than choosing PHP-style dynamic operator behavior.

MIR is then converted into [bytecode and linked](thp:guide.bytecodeLinking).
