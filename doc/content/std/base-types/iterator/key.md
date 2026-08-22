---
kind: method
id: std.baseTypes.Iterator::key
title: Iterator::key
summary: Returns the current typed key without advancing the cursor.
name: key
order: 3
typeParameters: []
parameters: []
returns:
  type: K
  description: The key of the current element.
errors:
  - description: The call fails when valid() is false; the concrete throwable type remains unsettled.
related: []
status: experimental
availability: proposed
notice: This member belongs to an experimental API contract and is not implemented in this repository.
version: "0.1"
owner: std.baseTypes.Iterator
visibility: public
modifiers: []
---

[`Iterator`](thp:std.baseTypes.Iterator)`::key()` returns the current typed key
without advancing the cursor.

## Behavior

Repeated calls return the same logical key until the cursor moves. The method
fails when `valid()` is false. An implementation may check its state directly;
it is not required to make a nested virtual call to `valid()`.

## Example

```thp
if ($iterator->valid()) {
    $key = $iterator->key();
}
```

## See also

- [`Iterator`](thp:std.baseTypes.Iterator)
- [`value()`](thp:std.baseTypes.Iterator::value)
