---
kind: method
id: std.dataStructures.ObjectStorage::get
title: ObjectStorage::get
summary: "Reads attached data or returns null for an absent object."
name: get
order: 2
typeParameters: []
parameters:
  - name: object
    type: "mixed"
    description: "Object identity to look up; non-object values are rejected."
returns:
  type: "?T"
  description: "Reads attached data or returns null for an absent object."
errors:
  - type: TypeError
    description: "The supplied key is not an object."
related: []
status: experimental
availability: implemented
notice: This experimental native method is implemented in the standalone compiler and VM.
version: "0.1"
owner: std.dataStructures.ObjectStorage
visibility: public
modifiers: []
---

Does not attach an absent object. If `T` allows null, use [`contains()`](thp:std.dataStructures.ObjectStorage::contains) to distinguish stored null from absence.

## See also

- [`ObjectStorage`](thp:std.dataStructures.ObjectStorage)
