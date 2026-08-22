# THP roadmap

THP is an experimental, statically typed, PHP-shaped language for greenfield
programs. This roadmap defines planned scope and dependency order. It does not
guarantee compatibility, release dates, or delivery. Detailed implementation
status is recorded in the
[implementation status](doc/content/guides/implementation-status.md).

The version identifiers express dependency order and do not define release
dates. Work on a dependent API begins after its prerequisite language and
library contracts are specified and implemented.

## v0.1.0: Experimental foundation

- Publish reproducible command-line archives for the supported desktop targets.
- Make implemented, partial, and proposed documentation visibly distinct.
- Keep native language and runtime specifications passing through `thp-test`.
- Define and test the project-discovery, OPcache-artifact, CLI, and C ABI version
  1 contracts required by experimental programs and embedding prototypes.

## v0.2.0: Contract and status reconciliation

- Correct stale stream notices.
- Remove claims that iterator transformations and typed collection functions
  have implementation support.
- Define a compact implementation matrix by symbol rather than documentation
  page.
- Specify whether `foreach` accepts both `Iterator` and `IteratorAggregate`.
- Define iterator rewind failures, mutation during iteration, exception
  propagation, generic variance, and key constraints.

Exit condition: the documentation describes one internally consistent iterator
protocol.

## v0.3.0: Generic interfaces

Implement the language prerequisite for the iterator contracts:

```thp
interface Traversable<K, V> {}
interface Iterator<K, V> extends Traversable<K, V> {}
interface IteratorAggregate<K, V> extends Traversable<K, V> {}
```

Required work:

- Generic interface syntax and AST representation.
- Type-argument arity and constraint checking.
- Substitution through inheritance and implementation.
- Override checking after substitution.
- Interface dispatch using instantiated signatures.
- Cross-module interface metadata and bytecode representation.

Tests must cover valid implementations, invalid type-argument arity,
incompatible methods, transitive inheritance, imports, and generic dispatch.

## v0.4.0: Iterator-object `foreach`

Add the first executable object protocol:

```text
IteratorAggregate::getIterator()
             ↓
rewind → valid → key/value → body → advance
```

Preserve existing direct vector and map iteration as the optimized path.

Required behavior:

- Evaluate the source exactly once.
- Infer the static key and value binding types.
- Invoke `IteratorAggregate::getIterator()` exactly once per traversal.
- Propagate iterator exceptions without interception or conversion.
- Preserve cleanup semantics for `break`, `continue`, `return`, `throw`,
  `using`, and `finally`.
- Call `key()` only for keyed loops and `value()` once per iteration.
- Fall back to the VM when the JIT cannot execute the protocol.

Exit condition: keyed and value-only `foreach` loops execute with user-defined
iterator and aggregate classes.

## v0.5.0: Native collection iterators

Implement:

- a `vector<T>` iterator as `Iterator<int, T>`;
- a `map<K, V>` iterator that preserves insertion order;
- `VectorIterator<T>`;
- `MapIterator<K, V>`;
- `EmptyIterator<K, V>`;
- `IteratorIterator<K, V>`;
- `iterator_count()`;
- `iterator_to_vector()`;
- `iterator_to_map()`.

Keep direct native `foreach` lowering for performance, but verify that its
observable behavior matches the iterator protocol.

## v0.6.0: Callables and collection operations

Closures and a typed callable model must precede callback-driven APIs.

Implement:

- closures and captured values;
- callable parameter typing and invocation;
- `vector_map()`;
- `vector_filter()`;
- `vector_slice()`;
- `vector_concat()`;
- `map_transform()`;
- `map_filter()`;
- `map_merge()`;
- `iterator_apply()`;
- `CallbackFilterIterator`;
- `FilterIterator`.

Specify key preservation, callback argument types, allocation failure,
exception propagation, and empty-collection inference for every operation.

## v0.7.0: Generators

Prerequisite: the v0.4.0 iterator-object `foreach` exit condition is satisfied.

Specify and implement:

- `yield $value`;
- `yield $key => $value`;
- generator return values;
- one-shot versus restartable behavior;
- exceptions thrown before and after suspension;
- cleanup of suspended frames;
- `finally` and `using` across suspension;
- behavior after exhaustion or explicit closure;
- whether generators accept values sent by callers and, if accepted, the send
  operation and its runtime semantics.

For v0.7.0, expose generators through `Iterator<K, V>`; do not introduce a
second traversal protocol.

## v0.8.0: Foundational standard library

Implement the following base contracts in dependency order:

1. `Option<T>`
2. `Countable`
3. `Stringable`
4. `MapAccess<K, V>`
5. `TraceLine` and throwable origin and trace APIs
6. PHP-inspired exception subclasses
7. A serialization format specification and conforming serialization APIs

## v0.9.0: Iterator adapters and data structures

Add these in dependency order:

1. `LimitIterator`, `InfiniteIterator`, and `AppendIterator`
2. Caching iterators
3. Recursive iterator interfaces
4. Recursive adapters
5. A fixed-size sequence type, queue, stack, and linked list
6. Heaps and priority queue
7. Object storage and a typed map wrapper

Every structure must implement the common iterator protocol instead of adding
special `foreach` handling.

## v0.10.0: Complete streams and filesystem

Finish the existing partial stream contract:

- `Stream` capability inspection;
- `write()` and `flush()`;
- relative seeking with `SeekFrom`;
- `Files::openWrite()` and `Files::openReadWrite()`;
- writable and read-write file handles;
- complete `OpenMode` and `WriteMode` support;
- local paths and `file://` through `Streams::open()`.

Then implement filesystem metadata and file iterators. This must precede
`DirectoryIterator`, `GlobIterator`, and their recursive adapters.

## v0.11.0: Async and extensions

Prerequisites: the callable, generator, and cleanup contracts are documented,
implemented, and covered by conformance tests.

Implement:

- coroutine scheduling and cancellation;
- `async()`, `await()`, `delay()`, `suspend()`, and `timeout()`;
- extension registration and native-call dispatch;
- bundled extension APIs;
- a regex extension, followed by `RegexIterator` and
  `RecursiveRegexIterator`;
- runtime autoload callbacks.

## Capability checkpoints

| Checkpoint                 | Version | Capability                                                     |
|----------------------------|---------|----------------------------------------------------------------|
| Iterable objects           | v0.4.0  | Generic interfaces and iterator-object `foreach`               |
| Collection toolkit         | v0.6.0  | Native iterators, closures, and core collection functions      |
| Lazy sequences             | v0.7.0  | Generators exposed through the iterator protocol               |
| Base library               | v0.9.0  | Foundational protocols, throwable traces, and core structures  |
| I/O library                | v0.10.0 | Complete streams, filesystem objects, and filesystem iterators |
| Concurrency and extensions | v0.11.0 | Async runtime and extension dispatch                           |

For every version, require positive, negative, and boundary unit tests plus
PHPT coverage. Update `doc/content/guides/implementation-status.md`, and mark an
API page `partial` or `implemented` only when its documented observable behavior
is executable.

## Later priorities

- Editor integration with syntax-aware diagnostics and source spans.
- A web/SAPI demonstration with a specified experimental request lifecycle.
- Standard-library APIs for JSON, text, time, HTTP, environment access, and
  process management.
- Expanded JIT coverage and benchmarks for incremental project compilation.

Language and standard-library proposals may change before implementation.
