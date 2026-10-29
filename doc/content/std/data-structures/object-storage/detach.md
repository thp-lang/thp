---
kind: method
id: std.dataStructures.ObjectStorage::detach
title: ObjectStorage::detach
summary: "Removes an object and its data if attached."
name: detach
order: 4
typeParameters: []
parameters:
  - name: object
    type: "mixed"
    description: "Object identity to look up; non-object values are rejected."
returns:
  type: "bool"
  description: "Removes an object and its data if attached."
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

Returns `true` when removed and `false` for an absent object. Releases this storage's references to object and data; other references can keep them alive. Other objects retain insertion order. Existing iterator snapshots still retain their captured objects.

## See also

- [`ObjectStorage`](thp:std.dataStructures.ObjectStorage)
