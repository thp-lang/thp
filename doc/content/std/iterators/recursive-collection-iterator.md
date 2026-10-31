---
kind: class
id: std.spl.RecursiveCollectionIterator
title: RecursiveCollectionIterator
summary: Traverses native vector and map trees through recursive entries.
name: RecursiveCollectionIterator
module: iterators
typeParameters: []
interfaces:
  - id: std.spl.RecursiveIterator
    arguments:
      - mixed
      - mixed
constants: []
properties: []
status: experimental
availability: proposed
notice: This experimental native recursive collection cursor is proposed and unavailable at runtime.
version: "0.1"
---

`RecursiveCollectionIterator` is a non-generic initial contract implementing
[`RecursiveIterator<mixed, mixed>`](thp:std.spl.RecursiveIterator). Its
[`__construct(mixed $values)`](thp:std.spl.RecursiveCollectionIterator::__construct)
accepts a native vector or map and throws `TypeError` for every other root,
including objects, scalars, and null. Explicit type arguments are not accepted.

## Keys, values, and shape

The cursor visits the root's elements, never the root container itself. Vector
keys remain their integer offsets; map keys keep their original type, identity,
and insertion order without coercion. Nothing flattens, reindexes, or converts a
map to a vector. An empty root is immediately invalid.

[`value()`](thp:std.baseTypes.Iterator::value) returns a
[`RecursiveEntry<mixed, mixed>`](thp:std.spl.RecursiveEntry). Each entry retains
the original element as its logical value. For a vector or map element,
[`children()`](thp:std.spl.RecursiveEntry::children) returns a child
`RecursiveCollectionIterator` over that collection. An empty nested collection
has a non-null but empty child cursor. Scalars, null, and objects are leaves
with null children; object properties are never inspected. Nested vectors and
maps retain their distinct shapes in both entry values and child traversal.

This intentionally loses static key and value precision: `key()` returns `mixed`
and entry values are `mixed`. Consumers must narrow values before typed use.
The loss allows heterogeneous vector/map trees without inventing a recursive
generic type or coercing every level to one key type.

## Snapshots and cursor lifetime

Construction captures collection value snapshots, including nested collection
values, using native copy-on-write semantics. Later source mutations do not
change traversal. Object leaves and object map keys retain identity and are
strongly retained; snapshots do not clone their properties.

Descent is lazy: construction does not eagerly build descendant cursors. When
an entry is first materialized by `value()`, its optional child cursor is created
from the captured nested value without visiting that child's elements. Repeated
`value()` calls return the same entry object and child cursor until parent
advancement. Children start at their first entry and have independent positions;
advancing or rewinding a parent never moves a previously returned child.

[`rewind()`](thp:std.baseTypes.Iterator::rewind) starts a fresh traversal of the
captured root snapshot with fresh entries and descendant cursors. It does not
recapture the source. Retained old entries and children remain usable. The cursor
releases its snapshots and cached entry when no longer retained; returned entries
and children can keep their captured values alive independently.

## Failures and traversal boundaries

The accepted input domain is finite acyclic trees. Shared subcollections are
allowed: each occurrence gets an independent child cursor. Only a collection
that repeats along the current ancestor path is a cycle. Detect such cycles on
lazy descent when `value()` would create the offending child cursor, throwing
`ValueError` at that call without advancing the parent. Do not reject shared
subcollections with a global visited set. This boundary applies even if ordinary
native collection construction cannot currently express a cycle.

The five cursor methods are inherited from [`Iterator`](thp:std.baseTypes.Iterator):
[`rewind()`](thp:std.baseTypes.Iterator::rewind),
[`valid()`](thp:std.baseTypes.Iterator::valid),
[`key()`](thp:std.baseTypes.Iterator::key),
[`value()`](thp:std.baseTypes.Iterator::value), and
[`advance()`](thp:std.baseTypes.Iterator::advance).
`key()` and `value()` fail when exhausted; the concrete exhausted-access throwable
remains unsettled as in the shared cursor contract. Advancing an exhausted cursor
leaves it exhausted. Allocation failures propagate from snapshot or entry creation.
`foreach` rewinds the direct cursor before traversal and uses the same shared
cursor protocol as other iterators.

## Proposed example

Unavailable at runtime until this proposal is implemented:

```thp
<?thp
$tree = new RecursiveCollectionIterator({"team" => ["Ada", "Lin"]});
foreach ($tree as $key => $entry) {
    echo $key; // team: original map key.
    $children = $entry->children();
    if ($children !== null) {
        foreach ($children as $offset => $child) {
            echo $offset; // 0, then 1: original vector offsets.
            echo $child->value(); // Ada, then Lin.
        }
    }
}
```

## See also

- [`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)
- [PHP migration placeholder](thp:std.spl.RecursiveArrayIterator)
- [Iterator catalog](thp:std.iterators)
