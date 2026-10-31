---
kind: method
id: std.dataStructures.ObjectStorage::getIterator
title: ObjectStorage::getIterator
summary: "Captures attached objects in insertion order."
name: getIterator
order: 6
typeParameters: []
parameters: []
returns:
  type: "Traversable<int, mixed>"
  description: "Captures attached objects in insertion order."
errors: []
related: []
status: experimental
availability: implemented
notice: This experimental native method is implemented in the standalone compiler and VM.
version: "0.1"
owner: std.dataStructures.ObjectStorage
visibility: public
modifiers: []
---

Returns a fresh [`VectorIterator`](thp:std.spl.VectorIterator) of objects, with dense zero-based integer keys. It yields objects rather than their attached data; use [`get()`](thp:std.dataStructures.ObjectStorage::get) to read current data. Later attachments and detachments do not change the captured membership. The cursor strongly retains its objects and shares their identity and property mutations. Each `foreach` obtains a fresh cursor.

## See also

- [`ObjectStorage`](thp:std.dataStructures.ObjectStorage)
