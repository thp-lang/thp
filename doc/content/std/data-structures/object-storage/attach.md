---
kind: method
id: std.dataStructures.ObjectStorage::attach
title: ObjectStorage::attach
summary: "Attaches data to an object identity."
name: attach
order: 1
typeParameters: []
parameters:
  - name: object
    type: "mixed"
    description: "Object identity to look up; non-object values are rejected."
  - name: data
    type: "T"
    description: "Data to retain with the object; required even for nullable T."
returns:
  type: "void"
  description: "This method does not return a value."
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

A new object appends in insertion order. Reattaching the same object replaces its data without moving it. Distinct objects remain distinct even if their properties match. Both object and data are strongly retained. Existing object snapshots are unchanged.

## See also

- [`ObjectStorage`](thp:std.dataStructures.ObjectStorage)
