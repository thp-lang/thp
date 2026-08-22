---
kind: method
id: std.baseTypes.Iterator::value
title: Iterator::value
summary: Returns the current typed value without advancing the cursor.
name: value
order: 4
typeParameters: []
parameters: []
returns:
  type: V
  description: The value of the current element.
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

[`Iterator`](thp:std.baseTypes.Iterator)`::value()` returns the current typed
value without advancing the cursor.

## Behavior

Repeated calls return the same logical value until the cursor moves. The method
fails when `valid()` is false. An implementation may check its state directly;
it is not required to make a nested virtual call to `valid()`.

## Example

```thp
if ($iterator->valid()) {
    $value = $iterator->value();
}
```

## See also

- [`Iterator`](thp:std.baseTypes.Iterator)
- [`key()`](thp:std.baseTypes.Iterator::key)
