---
kind: method
id: std.spl.RecursiveEntry::__construct
title: RecursiveEntry::__construct
summary: Pairs a value with its optional child iterator.
name: __construct
order: 0
typeParameters: []
parameters:
  - name: value
    type: T
    description: Logical value of the entry.
  - name: children
    type: ?RecursiveIterator<K, T>
    description: Child cursor, or null for a leaf.
    default: "null"
returns:
  type: void
  description: Creates the entry.
errors: []
related: []
status: experimental
availability: implemented
notice: This constructor is executable in the reference VM.
version: "0.1"
owner: std.spl.RecursiveEntry
visibility: public
modifiers: []
---

`RecursiveEntry<K, T>` retains the value and child cursor passed at construction.
