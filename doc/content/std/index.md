---
kind: module
id: std.index
title: Standard library
summary: The intended shape of THP's standard library and its evolving contracts.
module: standard-library
order: 0
status: experimental
availability: proposed
notice:
  This section defines THP's intended library contracts. Async, autoloading, Base,
  SPL, and typed collections are not presented as available runtime APIs in this
  documentation checkout.
---

## Async

[Async](thp:std.async) provides cooperatively scheduled coroutines without colored
functions. Register the [built-in scheduler](thp:std.async.scheduler_register_default)
or a [custom scheduler](thp:std.async.scheduler_register), start work with
[`async()`](thp:std.async.async), and retrieve results with
[`await()`](thp:std.async.await).

Use [`scheduler_is_registered()`](thp:std.async.scheduler_is_registered) when
bootstrap code needs to preserve a scheduler registered by the application.

[`Coroutine<T>`](thp:std.async.Coroutine) carries the typed result,
[`Completable<T>`](thp:std.async.Completable) defines the common one-shot contract,
and [SchedulerInterface](thp:std.async.SchedulerInterface) defines the contract for
custom scheduler implementations. [`delay()`](thp:std.async.delay) and
[`suspend()`](thp:std.async.suspend) yield cooperatively; [`timeout()`](thp:std.async.timeout)
limits an outstanding wait.

## Autoloading

[Autoloading](thp:std.dataStructures) defines a PHP-inspired, process-wide queue of
callbacks that load type declarations on demand. Use
[`spl_autoload_register()`](thp:std.spl.spl_autoload_register) and
[`spl_autoload_unregister()`](thp:std.spl.spl_autoload_unregister) to
manage loaders,
[`spl_autoload_functions()`](thp:std.spl.spl_autoload_functions) to inspect
them, and [`spl_autoload_call()`](thp:std.spl.spl_autoload_call) to start a
lookup explicitly.

The section also documents the configurable
[`spl_autoload()`](thp:std.spl.spl_autoload) default loader.

## PHP-inspired library categories

[`Exceptions`](thp:std.exceptions) documents the proposed exception
hierarchy. [`Iterators`](thp:std.iterators) covers adapters and recursive
traversal. [`Filesystem`](thp:std.filesystem) documents filesystem metadata,
files, temporary files, and locking results.
[`Data structures`](thp:std.dataStructures) contains generic containers,
observer contracts, autoloading, and object utilities.

## Streams

[`Streams`](thp:std.streams) defines typed native handles, deterministic
cleanup, capability interfaces, file and memory factories, and a dynamic URI
compatibility bridge.

## Bundled extensions

[Bundled extensions](thp:std.extensions) reserves reference routes for the
optional extensions shipped in PHP's source distribution. Every entry is
currently a design placeholder rather than an implemented compatibility
promise.

## Base

[Base](thp:std.baseTypes) contains THP-native foundational value contracts:
[`Option`](thp:std.baseTypes.Option) represents a value that may be absent, and
[`TraceLine`](thp:std.baseTypes.TraceLine) represents a typed throwable stack frame.

Engine-defined interfaces, exceptions, and attributes are documented in the
Language Reference, under
predefined interfaces and classes,
predefined exceptions, and
predefined attributes.

## Typed collections

THP defines `vector<T>` and insertion-ordered `map<K, V>` as native values with
generic element and key constraints. `[]` creates a vector,
`{key => value}` creates a map, and both support bracket access.

Collection operations use global functions prefixed by their native input
shape rather than methods or PHP's `array_*` names. The proposed vector family
includes `vector_map()`, `vector_filter()`, `vector_slice()`, and
`vector_concat()`. The proposed map family includes `map_transform()`,
`map_filter()`, and `map_merge()`. These helpers are all proposed.
`count(string|vector<T>|map<K, V>): int` is the only executable general
collection function in this checkout and reads length without traversal state.
It is distinct from proposed
[`iterator_count()`](thp:std.spl.iterator_count), which consumes an explicit
iterator from its current cursor without rewinding.
The executable `foreach` implementation currently works with native vectors
and maps, preserves single evaluation and keys, and supports `break` and
`continue`. The proposed object protocol uses `Iterator<K, V>` for both vector
and map iterators; iterator-object traversal remains unimplemented.

Native collection storage is intended to let the compiler and VM lower
construction, indexing, mutation, and iteration directly instead of wrapping
storage in ordinary generic objects. Physical storage remains an implementation
detail.

Typed collection errors can be caught with `try`/`catch`; uncaught runtime
failures are deterministic.

The broader standard library is not yet defined. There is no stable API for
text, files, time, networking, SPL, or package-provided libraries.
