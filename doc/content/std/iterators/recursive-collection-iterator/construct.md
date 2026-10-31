---
kind: method
id: std.spl.RecursiveCollectionIterator::__construct
title: RecursiveCollectionIterator::__construct
summary: Captures a native collection tree for lazy recursive traversal.
name: __construct
order: 0
typeParameters: []
parameters:
  - name: values
    type: mixed
    description: Native vector or map whose elements form the traversal root.
returns:
  type: void
  description: This method does not return a value.
errors:
  - type: TypeError
    description: The root is not a native vector or map.
related: []
status: experimental
availability: proposed
notice: This experimental method is proposed and unavailable at runtime.
version: "0.1"
owner: std.spl.RecursiveCollectionIterator
visibility: public
modifiers: []
---

Accepts only a native vector or map, preserving shape, original keys, and
insertion order. Captures collection value snapshots without eagerly visiting
descendants; starts on the first root element or exhausted for an empty root.
It never yields the root container itself. Objects are valid leaf elements but
invalid roots. The class takes no generic arguments and exposes
`RecursiveIterator<mixed, mixed>`.

Each current value is a `RecursiveEntry<mixed, mixed>` retaining its original
element and, for a nested vector or map, an independent child cursor. Empty
nested collections have empty child cursors. Objects are strongly retained
leaves and their properties are not inspected. Repeated current access returns
the same entry until advancement; rewind traverses the captured snapshot afresh.
See the class contract for snapshot lifetime, lazy ancestor-cycle detection,
shared subcollections, exhausted access, and fresh descendant cursors.

## See also

- [`RecursiveCollectionIterator`](thp:std.spl.RecursiveCollectionIterator)
