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

## Availability matrix

`implemented` means the documented behavior is executable in this checkout.
`partial` means only the limitation stated in the final column executes.
`proposed` means the symbol is documentation-only and must be rejected by the
compiler if used as though it were available.

| Symbol or syntax                                                                  | Availability | Executable boundary                                                                                              |
| --------------------------------------------------------------------------------- | ------------ | ---------------------------------------------------------------------------------------------------------------- |
| `<?thp`, comments, blocks, statements                                             | implemented  | UTF-8 source and ASCII identifiers                                                                               |
| `int`, `float`, `bool`, `string`, `null`, `void`, `mixed`, nullable types, unions | implemented  | `string` values are arbitrary bytes at runtime                                                                   |
| `vector<T>`, `map<K, V>`                                                          | implemented  | Literals, indexing, variable-rooted element assignment, COW values, and direct traversal                         |
| Variables and functions                                                           | implemented  | Typed parameters/returns, constant defaults, named and variadic arguments, calls, and recursion                  |
| `if`, `match`, `while`, `for`, `return`, `echo`                                   | implemented  | Conditions require `bool`; output supports `string`, `int`, `float`, and `bool`                                  |
| `foreach (vector<T>)`, `foreach (map<K, V>)`                                      | implemented  | Native collections only; the source is evaluated once and traversal uses its captured COW snapshot               |
| `break`, `continue`                                                               | partial      | Level one only; numeric levels are rejected                                                                      |
| Scalar operators                                                                  | partial      | Checked arithmetic, matching-type `==`, comparison, boolean short-circuiting, concatenation, and null coalescing |
| Classes and interfaces                                                            | partial      | Nominal classes and non-generic, methods-only interfaces; generic interfaces remain proposed                     |
| Traits                                                                            | implemented  | Compile-time composition, conflict selection, aliases, and visibility/finality adaptation                        |
| `Throwable`, `Exception`, `Error`, `UnhandledMatchError`                          | implemented  | Sealed throwable root, typed catches, common accessors, suppression, and deterministic uncaught failures         |
| `try`, `catch`, `finally`, `throw`, `using`                                       | implemented  | Ordered subtype catches and cleanup-preserving control transfer                                                  |
| Namespaces, imports, and project autoload discovery                               | implemented  | Semicolon namespaces and deterministic configured source maps; no runtime include/autoload callbacks             |
| OPcache, frozen projects, metrics, embedding, C ABI                               | implemented  | Cache/bytecode formats and C ABI remain version 1                                                                |
| Cranelift JIT                                                                     | partial      | Safe scalar subset; automatic mode falls back to the VM                                                          |

### Collection and iterator symbols

| Symbol                                                                 | Availability | Input and cursor behavior                                                                         |
| ---------------------------------------------------------------------- | ------------ | ------------------------------------------------------------------------------------------------- |
| `count(string\|vector<T>\|map<K, V>): int`                             | implemented  | Reads the value's byte or collection length; it does not consume, move, or create traversal state |
| `Traversable<K, V>`                                                    | proposed     | Invariant marker interface; `K` has no additional constraint                                      |
| `Iterator<K, V>`                                                       | proposed     | Invariant cursor interface extending `Traversable<K, V>`                                          |
| `IteratorAggregate<K, V>`                                              | proposed     | Invariant aggregate interface; `getIterator()` returns `Traversable<K, V>`                        |
| `foreach (Traversable<K, V>)`                                          | proposed     | Iterator-object dispatch and execution are not implemented                                        |
| `iterator_count<K, V>(Iterator<K, V>): int`                            | proposed     | Counts from the current cursor through exhaustion, advances it, and never rewinds                 |
| `iterator_apply()`                                                     | proposed     | Callback-driven consuming traversal is not implemented                                            |
| `iterator_to_vector()`, `iterator_to_map()`                            | proposed     | Consuming iterator conversions are not implemented                                                |
| `vector_map()`, `vector_filter()`, `vector_slice()`, `vector_concat()` | proposed     | Native-vector transformations are not implemented                                                 |
| `map_transform()`, `map_filter()`, `map_merge()`                       | proposed     | Native-map transformations are not implemented                                                    |
| Iterator adapters and recursive iterators                              | proposed     | All classes and members in the iterator module are documentation-only                             |

[`count()`](thp:std.baseTypes) and
[`iterator_count()`](thp:std.spl.iterator_count) are separate functions, not
aliases or overloads. The first inspects an existing native value without a
cursor; the second is a future consuming operation on an explicit iterator.

### Stream symbols

| Symbol                                                      | Availability | Executable boundary                                                                                  |
| ----------------------------------------------------------- | ------------ | ---------------------------------------------------------------------------------------------------- |
| `MemoryStream::open()`, `TempStream::open()`                | implemented  | Binary-safe memory streams and one-time temporary-file spill                                         |
| `Files::openRead()`                                         | implemented  | Existing local files opened read-only                                                                |
| `ReadableStream::read()`, `readAll()`, `eof()`              | implemented  | Includes argument, EOF, limit, and closed-handle behavior                                            |
| `WritableStream::writeAll()`                                | implemented  | Memory and temporary streams only                                                                    |
| `Closeable::close()`, `isClosed()`                          | implemented  | Shared, idempotent close state and deterministic `using` cleanup                                     |
| `SeekableStream::tell()`                                    | implemented  | Current absolute byte position                                                                       |
| `SeekableStream::seek()`                                    | partial      | One absolute position from the start; the executable form is `seek(int): void`                       |
| `OpenMode`                                                  | partial      | Only `Read`, `Write`, and `ReadWrite` names exist; usable URI combinations are narrower              |
| `Streams::open()`                                           | partial      | `php://memory`, `php://temp[/maxmemory:N]`, and read-only shared `thp:/input`; no paths or `file://` |
| Stream exception classes                                    | implemented  | Runtime-produced typed stream failures                                                               |
| `OpenStreamException::getTarget()`, `getSystemCode()`       | implemented  | Target and platform-code accessors                                                                   |
| `OpenStreamException::__construct()`                        | partial      | Runtime construction is supported; the documented public four-argument constructor is not            |
| `Stream::isReadable()`, `isWritable()`, `isSeekable()`      | proposed     | Capability inspection methods are not implemented; use `instanceof`                                  |
| `WritableStream::write()`, `flush()`                        | proposed     | Partial writes and explicit flushing are not implemented                                             |
| `SeekFrom`, relative/end seeking                            | proposed     | `Current` and `End` origins are not accepted by executable `seek()`                                  |
| `Files::openWrite()`, `Files::openReadWrite()`, `WriteMode` | proposed     | Writing factories and modes are not implemented                                                      |
| `WritableFileStream`, `ReadWriteFileStream`                 | proposed     | Writable and read-write file handles are not implemented                                             |

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
