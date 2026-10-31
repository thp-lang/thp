---
kind: method
id: std.spl.VectorIterator::__construct
title: VectorIterator::__construct
summary: Captures a vector snapshot and starts at its first offset.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: values
    type: vector<T>
    description: Vector to snapshot.
returns:
  type: void
  description: Initializes the cursor.
errors:
  - description: An incompatible argument is rejected by the type checker.
related: []
status: experimental
availability: implemented
notice: This experimental native constructor is implemented in the standalone compiler and VM.
version: "0.5"
owner: std.spl.VectorIterator
visibility: public
modifiers: []
---

`new VectorIterator($values)` infers `T` from a typed vector argument. An empty
vector requires a type context or explicit generic arguments.
