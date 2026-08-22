---
kind: method
id: std.baseTypes.Iterator::valid
title: Iterator::valid
summary: Reports whether the cursor identifies an element.
name: valid
order: 2
typeParameters: []
parameters: []
returns:
  type: bool
  description: True when key() and value() may read the current element; otherwise false.
errors: []
related: []
status: experimental
availability: proposed
notice: This member belongs to an experimental API contract and is not implemented in this repository.
version: "0.1"
owner: std.baseTypes.Iterator
visibility: public
modifiers: []
---

[`Iterator`](thp:std.baseTypes.Iterator)`::valid()` reports whether the cursor
currently identifies an element.

## Behavior

This method does not move the cursor. It returns `false` for an empty iterator,
before a successful positioning operation, and after exhaustion. Repeated
calls return the same result until `rewind()` or `advance()` changes the cursor.

## Example

```thp
while ($iterator->valid()) {
    echo $iterator->value();
    $iterator->advance();
}
```

## See also

- [`Iterator`](thp:std.baseTypes.Iterator)
- [`advance()`](thp:std.baseTypes.Iterator::advance)
