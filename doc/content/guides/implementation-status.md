---
kind: guide
id: guide.implementationStatus
title: Implementation status
summary: Distinguishes the executable THP bytecode-interpreter core from proposed language contracts.
nav:
  section: learn
  order: 15
status: experimental
availability: implemented
notice: >-
  This matrix describes the current checkout. Executable behavior remains
  experimental and may change with its language contract.
---

THP now has an initial standalone pipeline:

```text
UTF-8 source → spanned AST → typed HIR → CFG MIR → verified bytecode → VM
                                                                  ↘ Cranelift JIT
```

`thp inspect` exposes every intermediate form. `--metrics=human` and
`--metrics=json` report phase time, allocations, retained tracked bytes, peak
tracked live bytes, item counts, bytecode size, and executed VM instructions.
Cache lookup/publication and JIT compilation have separate metric stages.
Project compilation additionally measures module discovery, interface
extraction, incremental/cache work, linking, and prepared execution.

## Executable core

The current parser, type checker, and VM support:

- the required `<?thp` opening tag, ASCII identifiers, comments, blocks, and
  semicolon-terminated statements;
- `int`, `float`, `bool`, arbitrary-byte runtime `string`, `null`, `void`,
  `mixed`, `vector<T>`, `map<K, V>`, nullable types, and unions;
- inferred or annotated variables, assignment, functions with typed parameters
  and returns, constant defaults, named and variadic arguments, calls,
  recursion, `return`, `echo`, `if`/`elseif`/`else`, `while`, full-clause
  `for`, native-collection `foreach`, and level-one `break`/`continue`;
- scalar arithmetic, comparison, boolean short-circuiting, concatenation, and
  null coalescing; `echo` and concatenation share canonical output conversion
  for `string`, `int`, `float`, and `bool`, while `null` requires an explicit
  fallback;
- vector and insertion-ordered map literals, typed indexing, nested
  variable-rooted element assignment, direct native iteration, `count()`, and
  `var_dump()`;
- nominal classes and non-generic, methods-only interfaces with transitive
  single inheritance, multiple implemented interfaces, strict overrides,
  abstract/final validation, lexical visibility, flattened typed properties,
  inherited and explicit parent constructors, virtual/interface dispatch,
  direct `self::`/`parent::` calls, and late-static `static::` dispatch;
- compile-time traits with nested use, properties, abstract/concrete
  instance/static methods, conflict selection, aliases, visibility/finality
  adaptation, and consumer-specialized bodies;
- sealed `Throwable`, user-defined `Exception` descendants, subtype catches,
  ordered handlers, `finally` unwinding and transfer replacement, common
  message/code/previous/suppressed state, catchable `UnhandledMatchError`,
  `match` expressions, and suppressed `using` cleanup failures;
- semicolon-style namespaces, separate type/function imports, fully qualified
  references, deterministic `[autoload]` project discovery, cross-file
  functions and nominal declarations, legal declaration SCCs, frozen linked
  programs, and reusable prepared projects;
- binary-safe memory/temporary streams, shared cursor and close state,
  capability `instanceof`, URI factories, read-only file opening, typed stream
  failures, request-bound `thp:/input`, deterministic `using` cleanup, and
  logical handle limits;
- signed 64-bit checked integer arithmetic, structured compile diagnostics,
  structured runtime failures, streamed host output, managed-heap/input/time/
  stack/handle limits, and an optional VM instruction limit.

Empty collection literals require an expected generic type:

```thp
<?thp

$users: vector<string> = [];
$scores: map<string, int> = {};
```

The interpreter uses 16-byte general value slots. Heap strings, collections,
and objects use non-atomic request-thread reference counting; vector and map
mutation detaches shared storage before writing, while object aliases observe
the same property mutations. A PHP-style buffered trial-deletion pass reclaims
unreachable collection, object, and exception cycles at request-safe points
and final teardown.

## OPcache and JIT

`thp run --opcache=PATH` uses a persistent, content-addressed bytecode cache.
The key covers source bytes, compiler and bytecode versions, effective
configuration, bytecode schema, OS, and architecture. Cache artifacts are atomically published
and fully decoded and verified before execution; missing or corrupt entries
compile again.

For projects, `thp cache-warm --opcache=PATH FILE` publishes `.thpi`
interfaces, `.thpo` module objects, one `.thpbc` linked program, and a `.thpm`
manifest last. `thp run --frozen --opcache=PATH FILE` validates configuration,
compiler/format identity, entry selection, and the verified program without
scanning mapped source directories.

`--engine=jit` selects a real Cranelift native-code tier. Its first safe subset
supports straight-line scalar functions, locals, direct calls, comparisons,
and boolean/null tests. `--engine=auto` uses this tier only for programs it can
execute with identical semantics and falls back to the VM for heap operations,
control-flow graphs, checked arithmetic, output, objects, method dispatch,
exception regions, cleanup instructions, or instruction limits.

## Deliberately rejected or pending

The current executable subset rejects unsupported syntax instead of inheriting
PHP behavior. Pending work includes:

- closures, call-site argument unpacking, by-reference parameters, general
  assignment expressions, property-rooted collection mutation, iterator-object
  `foreach`, and numeric `break`/`continue` levels;
- generic and multiple-parent interfaces, interface state, trait constants,
  static properties, property hooks, magic methods, anonymous classes, enums,
  reflection, and flow narrowing after `instanceof`;
- global constants, dynamic names, runtime includes/autoload callbacks,
  attributes, generators, reflection, and cooperative async;
- the broader standard library, extension registration/dispatch, concrete
  FastCGI and web-server SAPI adapters, relocatable module code generation,
  dependency-minimal incremental invalidation, broader JIT coverage, hotness
  tiering, and shared native-code caching.

`==` currently accepts only matching static operand types and performs no PHP
type juggling. Boolean conditions require `bool`; general truthiness is not
implemented. Source files are UTF-8 even though runtime strings may contain
arbitrary bytes.

## Validation boundary

Bytecode is versioned, serialized, bounds checked, type checked, and verified
before execution. Compile phases return diagnostics without printing or
terminating. The CLI alone renders diagnostics and writes program output.

`thp-embed` provides a safe in-process engine and request/response SAPI trait.
`crates/abi/include/thp.h` exposes the version-one C embedding ABI with opaque
engines and prepared projects, size-versioned runtime limits, synchronous
input/output and module enumeration/loading callbacks, legacy captured output,
binary-safe owned buffers, explicit release, panic containment, and versioned
extension/host tables. Loading and invoking third-party extensions and concrete
server adapters remain pending.

The resource and core-language PHPT files run through the THP compiler and VM
as part of `cargo test --workspace`. They must not be passed to PHP.
