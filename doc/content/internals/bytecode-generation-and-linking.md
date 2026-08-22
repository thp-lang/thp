---
kind: guide
id: guide.bytecodeLinking
title: Bytecode generation and linking
summary: See how MIR becomes a complete, versioned register-bytecode program.
nav:
  section: internals
  order: 80
status: experimental
availability: implemented
notice: >-
  THP bytecode is versioned but remains internal to the matching compiler and
  runtime.
---

`thp-bytecode` lowers MIR into the only program representation accepted by the
VM. A bytecode `Program` owns function and class descriptors, entry identity,
blocks, instructions, terminators, exception handlers, and the source spans
needed at runtime.

```thp
<?thp
function greet(string $name): string {
    return "Hello, " . $name;
}

echo greet("Ada");
```

Current `thp inspect --emit=bytecode greet.thp` output includes:

```text
THP bytecode v1 entry=#0
function #0 <main> locals=0 registers=2 -> void
  block 0:
    r0 = Constant(String([65, 100, 97]))
    r1 = Call { callee: Function(FunctionId(1)), arguments: [Register(0)] }
    Print(Register(1))
    Return(None)
function #1 greet locals=1 registers=3 -> string
  block 0:
    r0 = Constant(String([72, 101, 108, 108, 111, 44, 32]))
    r1 = LoadLocal(LocalId(0))
    r2 = Binary { op: Concatenate, left: Register(0), right: Register(1) }
    Return(Some(Register(2)))
```

Strings display as byte arrays because THP runtime strings are binary-safe.

## Single sources and projects

For a single source file, bytecode lowering creates the complete program after
MIR. For a project, module interfaces and bodies are compiled in dependency
order and linked into one program with stable function, class, property, and
method identities. The linked program has one selected entry function.

Persistent project caching distinguishes interfaces (`.thpi`), module objects
(`.thpo`), linked programs (`.thpbc`), and frozen manifests (`.thpm`). These
formats serve different validation and invalidation boundaries; they are not
renamed copies of one artifact.

The binary codec records an explicit bytecode schema version. Decoding performs
bounds checks, but successful decoding is still followed by semantic bytecode
verification.

## Design choices compared with PHP

PHP compiles each loaded script into Zend op arrays whose operands work with
dynamically tagged values. Late-bound declarations and runtime file loading fit
naturally into that per-script model.

THP chooses a register bytecode with declared local/register counts, typed
function and method descriptors, numeric class/member identities, explicit
handlers, and one linked entry point. Linking the project ahead of execution
lets calls and dispatch tables use compact verified IDs and lets frozen
deployments load one completed program. The cost is a distinct link step and
less freedom to add declarations after execution has begun.

Every generated or decoded program must pass
[bytecode verification](thp:guide.bytecodeVerification).
