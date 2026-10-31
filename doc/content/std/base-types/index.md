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
  The executable runtime implements the type guards and the core object,
  throwable, and stream contracts described by their individual notices. Other
  base-type APIs remain proposals.
---

Base contains foundational values that support other standard-library APIs.
Engine-defined interfaces and throwable classes belong to the
Language Reference
rather than this library section.

The [`is_string()`](thp:std.baseTypes.is_string),
[`is_int()`](thp:std.baseTypes.is_int),
[`is_float()`](thp:std.baseTypes.is_float),
[`is_null()`](thp:std.baseTypes.is_null),
[`is_numeric()`](thp:std.baseTypes.is_numeric),
[`is_vector()`](thp:std.baseTypes.is_vector), and
[`is_map()`](thp:std.baseTypes.is_map) runtime guards accept `mixed` and refine
a directly guarded local inside a positive `if` or `elseif` branch.

| Type                                                           | Description                                     |
| -------------------------------------------------------------- | ----------------------------------------------- |
| [`Throwable`](thp:std.baseTypes.Throwable)                     | Sealed interface for values accepted by throw.  |
| [`Exception`](thp:std.baseTypes.Exception)                     | Base class for application failures.            |
| [`Error`](thp:std.baseTypes.Error)                             | Base class for engine-detected language errors. |
| [`TypeError`](thp:std.baseTypes.TypeError)                     | Reports a dynamic argument type mismatch.       |
| [`ArgumentCountError`](thp:std.baseTypes.ArgumentCountError)   | Reports a dynamic argument count mismatch.      |
| [`UnhandledMatchError`](thp:std.baseTypes.UnhandledMatchError) | Reports a `match` with no selected arm.         |
| [`Option`](thp:std.baseTypes.Option)                           | Represents either one value or no value.        |
| [`Countable`](thp:std.baseTypes.Countable)                     | Supplies a non-negative count.                  |
| [`Stringable`](thp:std.baseTypes.Stringable)                   | Supplies a string representation.               |
| [`MapAccess`](thp:std.baseTypes.MapAccess)                     | Supplies explicit offset methods.               |
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
shape rather than methods or PHP's `array_*` names. The following operations
execute in the reference VM:

| Operation                                            | Callback arguments and result | Result keys                                  | Empty input inference                                |
| ---------------------------------------------------- | ----------------------------- | -------------------------------------------- | ---------------------------------------------------- |
| [`vector_map()`](thp:std.baseTypes.vector_map)       | `(T): U`                      | Dense `0..n-1`                               | `T` from callback, `U` from return type              |
| [`vector_filter()`](thp:std.baseTypes.vector_filter) | `(T): bool`                   | Dense `0..n-1`                               | `T` from callback                                    |
| [`vector_slice()`](thp:std.baseTypes.vector_slice)   | None                          | Dense `0..n-1`                               | Expected `vector<T>` required                        |
| [`vector_concat()`](thp:std.baseTypes.vector_concat) | None                          | Dense `0..n-1`                               | Other operand, or expected `vector<T>` if both empty |
| [`map_transform()`](thp:std.baseTypes.map_transform) | `(V, K): U`                   | Original keys and positions                  | `V`, `K` from callback; `U` from return type         |
| [`map_filter()`](thp:std.baseTypes.map_filter)       | `(V, K): bool`                | Retained keys and positions                  | `V`, `K` from callback                               |
| [`map_merge()`](thp:std.baseTypes.map_merge)         | None                          | First position of each key; right value wins | Other operand, or expected `map<K, V>` if both empty |

Each callback is called once per visited input in source order. Filter
callbacks must return `bool`; no truthiness conversion occurs. Inputs keep
their value semantics and are unchanged. A callback exception stops the
operation at that element and propagates without a partial result. Allocation
failure, including result growth or closure creation, stops with the runtime's
allocation error; no partial result is returned. Operations without callbacks
have no callback exception path, but still report allocation failure. An empty
result retains its statically determined generic types. An untyped empty
literal with no expected type is a compile error.

`count(string|vector<T>|map<K, V>|Countable): int` reads the native length or
dispatches `Countable::count()` without consuming traversal state. Implemented
[`iterator_count()`](thp:std.spl.iterator_count) instead accepts an
`Iterator<K, V>`, counts from its current cursor through exhaustion, advances
it, and does not rewind; the two names are neither aliases nor overloads.
The executable `foreach` implementation works with native vectors and maps and
with `Traversable<K, V>` objects. It preserves single evaluation and keys and
supports `break` and `continue`.

Native collection storage is intended to let the compiler and VM lower
construction, indexing, mutation, and iteration directly instead of wrapping
storage in ordinary generic objects. Physical storage remains an implementation
detail.

[`serialize()`](thp:std.baseTypes.serialize) and
[`unserialize()`](thp:std.baseTypes.unserialize) implement the bounded
[THP serialization format](thp:guide.serializationFormat) for scalar and native
collection values. Object serialization hooks remain proposed.

Typed collection errors can be caught with `try`/`catch`; uncaught runtime
failures are deterministic.

The broader standard library is not yet defined. There is no stable API for
text, files, time, networking, SPL, or package-provided libraries.
