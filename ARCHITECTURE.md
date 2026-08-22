# THP implementation architecture

THP is a standalone language implementation. It does not emit PHP and it does
not execute through the PHP engine.

## Compiler pipeline

The executable pipeline is intentionally one-way:

```text
sources -> AST units -> interfaces/index -> resolved module graph
        -> typed HIR units -> MIR objects -> link -> verified bytecode
        -> VM -> JIT
```

Each arrow is an explicit API boundary. Syntax nodes preserve source spans.
`thp-modules` owns deterministic source discovery, canonical export names,
dependency kinds, and cyclic declaration groups. HIR owns resolved names and
semantic types. MIR owns control flow and cleanup.
Bytecode is the only interpreter input and must pass verification before a VM
can execute it. The JIT must implement the same observable behavior as the VM.

Compiler phases return structured diagnostics. Only presentation layers such
as the CLI and a SAPI may render or print them.

## Runtime boundaries

Request values are isolated from persistent engine, module, and cache state.
The initial runtime executes one request on one VM thread. Heap values use
non-atomic reference counting and copy-on-write collection storage. Objects
share property mutation across aliases. A request-owned heap accounts managed
cells and payload capacity, buffers possible cycle roots, and uses trial
deletion to reclaim unreachable vector, map, object, and exception cycles.
Stream cells own logical handle leases and release native storage on explicit
close, last-reference cleanup, or request teardown.

VM requests carry instruction, execution-time, managed-heap, request-input,
logical-stack, and open-handle limits plus a request-local filesystem base.
Relative file operations resolve against that base; presentation layers must
not change the process working directory. Output is written synchronously to a
host sink and has no total-size quota. The explicit capture wrapper retains
output and counters on failure for legacy hosts and the PHPT runner.

The embedding boundary has two layers. `thp-embed` owns the safe request,
response, engine, prepared-project, and streaming SAPI traits. `thp-abi`
exposes version-one opaque engine/prepared-project handles, size-versioned
runtime limits, synchronous input/output and module-provider callbacks, owned
byte buffers, and extension/host function tables through
`crates/abi/include/thp.h`. Legacy captured run functions remain available.
Rust layouts are never part of that ABI, panics are caught before returning to
C, and every allocation has an explicit matching release function.

## Observability

Every public command can record wall time, allocation activity, retained tracked
bytes, and peak tracked live bytes for source loading, lexing, parsing, HIR,
MIR, bytecode generation, verification, VM execution, cache access, and JIT
compilation. The human format is for interactive use; the versioned JSON format
is for benchmark and CI ingestion.

IR dump formats are debugging interfaces. Bytecode and metrics formats carry
explicit schema versions and reject unsupported versions. Single-source
OPcache keys include source bytes, caller-provided configuration, compiler and
format versions, OS, and architecture. Project interfaces and objects also
include module and dependency hashes. Full resolved target and local
configuration coverage in project and frozen identities remains pending.
Single-source entries, project interfaces, objects, linked programs, and frozen
manifests use distinct extensions. Entries are atomically published; linked
programs are verified on load, and a frozen manifest is published only after
all referenced artifacts.

The first Cranelift tier accepts only straight-line scalar functions whose
operations cannot raise a THP runtime error. `auto` checks support before
compiling and otherwise uses the VM. Expanding native coverage must preserve
checked arithmetic, cleanup, exceptions, and instruction-limit behavior rather
than weakening them.

## Unsafe code

Unsafe Rust is allowed only where its performance or ABI purpose is explicit:
allocation tracking, compact runtime values, reference-counted heap primitives,
native extension adapters, bytecode dispatch, and executable JIT memory. Every
unsafe block must state the invariant that makes it valid and have focused
tests; ordinary compiler code remains safe Rust.
