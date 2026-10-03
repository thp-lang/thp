---
kind: method
id: std.spl.MapIterator::__construct
title: MapIterator::__construct
summary: Captures a map snapshot and starts at its first entry.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: values
    type: map<K, V>
    description: Map to snapshot.
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
owner: std.spl.MapIterator
visibility: public
modifiers: []
---

`new MapIterator($values)` infers `K` and `V` from a typed map argument. An
empty map requires a type context or explicit generic arguments.
