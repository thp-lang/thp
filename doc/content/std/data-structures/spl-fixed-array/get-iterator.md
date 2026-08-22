---
kind: method
id: std.spl.SplFixedArray::getIterator
title: SplFixedArray::getIterator
summary: Returns index-value entries in ascending order.
name: getIterator
order: 12
typeParameters: []
parameters: []
returns:
  type: Iterator<int, ?T>
  description: Returns index-value entries in ascending order.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.spl.SplFixedArray
visibility: public
modifiers: []
---

[`SplFixedArray`](thp:std.spl.SplFixedArray)`::getIterator()` returns index-value entries in ascending order.

## Behavior

Returns index-value entries in ascending order.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getIterator();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFixedArray`](thp:std.spl.SplFixedArray)
