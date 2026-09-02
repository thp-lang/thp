---
kind: module
id: std.baseTypes
title: Base types
summary: Foundational value types and contracts available to THP programs.
module: base-types
order: 20
status: experimental
availability: proposed
notice: >-
  The executable runtime implements the core object, throwable, and stream
  contracts described by their individual notices. Other base-type APIs remain
  proposals.
---

Base contains foundational values that support other standard-library APIs.
Engine-defined interfaces and throwable classes belong to the
Language Reference
rather than this library section.

| Type                                                           | Description                                     |
| -------------------------------------------------------------- | ----------------------------------------------- |
| [`Throwable`](thp:std.baseTypes.Throwable)                     | Sealed interface for values accepted by throw.  |
| [`Exception`](thp:std.baseTypes.Exception)                     | Base class for application failures.            |
| [`Error`](thp:std.baseTypes.Error)                             | Base class for engine-detected language errors. |
| [`UnhandledMatchError`](thp:std.baseTypes.UnhandledMatchError) | Reports a `match` with no selected arm.         |
| [`Option`](thp:std.baseTypes.Option)                           | Represents either one value or no value.        |
| [`TraceLine`](thp:std.baseTypes.TraceLine)                     | Represents one frame in a captured stack trace. |
| [`Iterator`](thp:std.baseTypes.Iterator)                       | Traverses typed keys and values with a cursor.  |
| [`IteratorAggregate`](thp:std.baseTypes.IteratorAggregate)     | Produces the next traversable layer.            |

## See also

- Predefined interfaces and classes
- Predefined exceptions

## Native typed collections

THP defines `vector<T>` and insertion-ordered `map<K, V>` as native values with
generic element and key constraints. `[]` creates a vector,
`{key => value}` creates a map, and both support bracket access.

Collection operations use global functions prefixed by their native input
shape rather than methods or PHP's `array_*` names. Proposed functions include
`vector_map()`, `vector_filter()`, `vector_slice()`, `vector_concat()`,
`map_transform()`, `map_filter()`, and `map_merge()`. Every transformation is
proposed. `count(string|vector<T>|map<K, V>): int` is the only executable
general collection function in this checkout. It reads the collection length
without consuming, moving, or creating traversal state. Proposed
[`iterator_count()`](thp:std.spl.iterator_count) instead accepts an
`Iterator<K, V>`, counts from its current cursor through exhaustion, advances
it, and does not rewind; the two names are neither aliases nor overloads.
The executable `foreach` implementation currently works with native vectors
and maps, preserves single evaluation and keys, and supports `break` and
`continue`. Iterator objects remain a proposal.

Native collection storage is intended to let the compiler and VM lower
construction, indexing, mutation, and iteration directly instead of wrapping
storage in ordinary generic objects. Physical storage remains an implementation
detail.

Typed collection errors can be caught with `try`/`catch`; uncaught runtime
failures are deterministic.

The broader standard library is not yet defined. There is no stable API for
text, files, time, networking, SPL, or package-provided libraries.
