---
kind: method
id: std.dataStructures.ObjectStorage::__construct
title: ObjectStorage::__construct
summary: "Creates an empty object-identity association."
name: __construct
order: 0
typeParameters: []
parameters: []
returns:
  type: "void"
  description: "This method does not return a value."
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

Specify `T` explicitly when context cannot infer the attached data type. Attached objects and data are strongly retained until detached or the storage is released, subject to other references.

## See also

- [`ObjectStorage`](thp:std.dataStructures.ObjectStorage)
