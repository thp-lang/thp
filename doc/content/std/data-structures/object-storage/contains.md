---
kind: method
id: std.dataStructures.ObjectStorage::contains
title: ObjectStorage::contains
summary: "Tests whether an object identity is attached."
name: contains
order: 3
typeParameters: []
parameters:
  - name: object
    type: "mixed"
    description: "Object identity to look up; non-object values are rejected."
returns:
  type: "bool"
  description: "Tests whether an object identity is attached."
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

Returns `true` even if attached data is null. Does not compare object properties or attach the object.

## See also

- [`ObjectStorage`](thp:std.dataStructures.ObjectStorage)
