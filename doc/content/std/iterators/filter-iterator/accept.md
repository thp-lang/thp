---
kind: method
id: std.spl.FilterIterator::accept
title: FilterIterator::accept
summary: Reports whether the current value passes the filter.
name: accept
order: 2
typeParameters: []
parameters:
  - name: value
    type: V
    description: Current inner value.
  - name: key
    type: K
    description: Current inner key.
returns:
  type: bool
  description: Reports whether the current value passes the filter.
errors:
  - description:
      Failures thrown by the callback or comparison operation propagate without
      being wrapped.
related: []
status: experimental
availability: implemented
notice: Concrete FilterIterator subclasses implement this abstract method.
version: "0.6"
owner: std.spl.FilterIterator
visibility: public
modifiers:
  - abstract
---

[`FilterIterator`](thp:std.spl.FilterIterator)`::accept()` reports whether the current value passes the filter.

## Behavior

Reports whether the current value passes the filter.

The adapter invokes `accept(V, K): bool` at most once for each current entry
until it advances. It preserves the original key when the method returns
`true`. An exception propagates and leaves the inner cursor at that entry.

## Example

```thp
$result = $instance->accept($value, $key);
```

The call uses the signature and defaults documented above.

## See also

- [`FilterIterator`](thp:std.spl.FilterIterator)
