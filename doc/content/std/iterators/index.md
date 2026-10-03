---
kind: module
id: std.iterators
title: Iterators
summary: Typed iterator adapters and recursive traversal.
module: iterators
order: 40
status: experimental
availability: partial
notice:
  Native collection iterators, EmptyIterator, IteratorIterator, OuterIterator,
  and three consuming functions are implemented. Other adapters remain proposed.
---

| Family                | Classes                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| --------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Composition           | [`AppendIterator`](thp:std.spl.AppendIterator), [`IteratorIterator`](thp:std.spl.IteratorIterator), [`MultipleIterator`](thp:std.spl.MultipleIterator)                                                                                                                                                                                                                                                                                                                     |
| Filtering             | [`FilterIterator`](thp:std.spl.FilterIterator), [`CallbackFilterIterator`](thp:std.spl.CallbackFilterIterator), [`RegexIterator`](thp:std.spl.RegexIterator)                                                                                                                                                                                                                                                                                                               |
| Position and lifetime | [`InfiniteIterator`](thp:std.spl.InfiniteIterator), [`LimitIterator`](thp:std.spl.LimitIterator), [`EmptyIterator`](thp:std.spl.EmptyIterator)                                                                                                                                                                                                                                                                                                                             |
| Caching               | [`CachingIterator`](thp:std.spl.CachingIterator)                                                                                                                                                                                                                                                                                                                                                                                                                           |
| Native collections    | [`VectorIterator`](thp:std.spl.VectorIterator), [`MapIterator`](thp:std.spl.MapIterator), and proposed `RecursiveCollectionIterator`                                                                                                                                                                                                                                                                                                                                       |
| Filesystems           | [`DirectoryIterator`](thp:std.spl.DirectoryIterator), [`FilesystemIterator`](thp:std.spl.FilesystemIterator), [`GlobIterator`](thp:std.spl.GlobIterator), [`RecursiveDirectoryIterator`](thp:std.spl.RecursiveDirectoryIterator)                                                                                                                                                                                                                                           |
| Recursive traversal   | [`ParentIterator`](thp:std.spl.ParentIterator), [`RecursiveCachingIterator`](thp:std.spl.RecursiveCachingIterator), [`RecursiveCallbackFilterIterator`](thp:std.spl.RecursiveCallbackFilterIterator), [`RecursiveFilterIterator`](thp:std.spl.RecursiveFilterIterator), [`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator), [`RecursiveRegexIterator`](thp:std.spl.RecursiveRegexIterator), [`RecursiveTreeIterator`](thp:std.spl.RecursiveTreeIterator) |

## See also

- [SPL reference](thp:std.dataStructures)
- [THP Iterator](thp:std.baseTypes.Iterator)
- [PHP SPL iterators](https://www.php.net/manual/en/spl.iterators.php)

## Iterator interfaces

The category also defines
[`OuterIterator`](thp:std.spl.OuterIterator),
[`RecursiveEntry`](thp:std.spl.RecursiveEntry),
[`RecursiveIterator`](thp:std.spl.RecursiveIterator), and
[`SeekableIterator`](thp:std.spl.SeekableIterator).

All iterators expose typed keys and values through
[`Iterator<K, V>`](thp:std.baseTypes.Iterator). Vector-backed iterators use
`int` keys; map-backed iterators preserve their declared key type. The cursor
protocol does not construct an option or entry object during ordinary
`foreach` traversal.

The executable object protocol evaluates the source once, calls `getIterator()`
once per aggregate layer, then calls `rewind()` on the direct iterator. Each
iteration is `valid() → value() → optional key() → body → advance()`.
`continue` advances; `break`, `return`, and a throw do not. Iterator failures
propagate unchanged through required `using` and `finally` cleanup. Native
collections retain their captured COW snapshot, while mutation of a delegated
iterator object remains visible according to that iterator's methods.

At a typed assignment, property, argument, or return boundary that requires a
compatible `Iterator<K, V>` or `Traversable<K, V>`, a vector or map creates a
fresh native iterator. Nullable targets and unions with one unambiguous
iterator branch are accepted. Passing an existing iterator preserves its
identity and current cursor. Raw collections have no cursor methods.

## Iterator functions

[`iterator_count()`](thp:std.spl.iterator_count),
[`iterator_to_vector()`](thp:std.spl.iterator_to_vector), and
[`iterator_to_map()`](thp:std.spl.iterator_to_map) consume an `Iterator<K, V>`
from its current cursor without rewinding. The vector conversion discards keys;
the map conversion preserves order and updates repeated keys at their first
position. [`iterator_apply()`](thp:std.spl.iterator_apply) remains proposed.
[`count()`](thp:std.baseTypes) reads a string, vector, or map length without
traversal state.

The PHP-derived `ArrayIterator`, `RecursiveArrayIterator`, and
`iterator_to_array()` pages remain migration-analysis placeholders. Their
`array` names are not accepted THP-native API names: THP has separate
`vector<T>` and `map<K, V>` types. Native collection iterator contracts use
`VectorIterator` and `MapIterator`; conversion functions name their result
shape explicitly.
