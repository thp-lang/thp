---
kind: method
id: std.dataStructures.MaxHeap::insert
title: MaxHeap::insert
summary: "Inserts a value for ordered extraction."
name: insert
order: 1
typeParameters: []
parameters:
  - name: value
    type: "T"
    description: "Value to store."
returns:
  type: "void"
  description: "This method does not return a value."
errors:
  - type: ValueError
    description: "The value is NaN."
  - description: "Values that do not support the required comparison fail at insertion; comparison failures propagate."
related: []
status: experimental
availability: implemented
notice: This experimental native method is implemented in the standalone compiler and VM.
version: "0.1"
owner: std.dataStructures.MaxHeap
visibility: public
modifiers: []
---

Validates the value against itself and every stored value using THP's greater-than comparison before storing it. Integers, non-NaN floats, and strings of mutually comparable types are supported; there is no PHP-style coercion or custom comparator. Rejection leaves the stored values unchanged. Existing snapshots remain unchanged.

## See also

- [`MaxHeap`](thp:std.dataStructures.MaxHeap)
