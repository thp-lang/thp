---
kind: module
id: std.iterators
title: Iterators
summary: Typed iterator adapters and recursive traversal.
module: iterators
order: 40
status: experimental
availability: proposed
notice:
  These future-facing adapters use THP's rewindable typed cursor protocol. They
  are not implemented in this checkout.
---

| Family                | Classes                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| --------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Composition           | [`AppendIterator`](thp:std.spl.AppendIterator), [`IteratorIterator`](thp:std.spl.IteratorIterator), [`MultipleIterator`](thp:std.spl.MultipleIterator)                                                                                                                                                                                                                                                                                                                     |
| Filtering             | [`FilterIterator`](thp:std.spl.FilterIterator), [`CallbackFilterIterator`](thp:std.spl.CallbackFilterIterator), [`RegexIterator`](thp:std.spl.RegexIterator)                                                                                                                                                                                                                                                                                                               |
| Position and lifetime | [`InfiniteIterator`](thp:std.spl.InfiniteIterator), [`LimitIterator`](thp:std.spl.LimitIterator), [`EmptyIterator`](thp:std.spl.EmptyIterator)                                                                                                                                                                                                                                                                                                                             |
| Caching               | [`CachingIterator`](thp:std.spl.CachingIterator)                                                                                                                                                                                                                                                                                                                                                                                                                           |
| Native collections    | Proposed `VectorIterator`, `MapIterator`, and `RecursiveCollectionIterator` contracts                                                                                                                                                                                                                                                                                                                                                                                      |
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

The proposed object protocol evaluates the source once, calls `getIterator()`
once per aggregate layer, then calls `rewind()` on the direct iterator. Each
iteration is `valid() → value() → optional key() → body → advance()`.
`continue` advances; `break`, `return`, and a throw do not. Iterator failures
propagate unchanged through required `using` and `finally` cleanup. Native
collections retain their captured COW snapshot, while mutation of a delegated
iterator object remains visible according to that iterator's methods.

## Iterator functions

[`iterator_apply()`](thp:std.spl.iterator_apply),
[`iterator_count()`](thp:std.spl.iterator_count), `iterator_to_vector()`, and
`iterator_to_map()` are all proposed consuming operations. None is available
in this checkout. `iterator_count()` accepts an `Iterator<K, V>`, starts at its
current cursor, advances through exhaustion, and never rewinds. It is separate
from executable [`count()`](thp:std.baseTypes), which accepts only a string,
vector, or map and reads its length without traversal state.

The PHP-derived `ArrayIterator`, `RecursiveArrayIterator`, and
`iterator_to_array()` pages remain migration-analysis placeholders. Their
`array` names are not accepted THP-native API names: THP has separate
`vector<T>` and `map<K, V>` types. Native collection iterator contracts will use
`VectorIterator` and `MapIterator`; collection conversion functions will name
their result shape explicitly.
