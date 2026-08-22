---
kind: method
id: std.spl.SplFixedArray::fromVector
title: SplFixedArray::fromVector
summary: Copies contiguous vector values into fixed slots.
name: fromVector
order: 4
typeParameters: []
parameters:
  - name: values
    type: vector<T>
    description: Initial values consumed or stored by the operation.
returns:
  type: SplFixedArray<T>
  description: Copies contiguous vector values into fixed slots.
errors:
  - description:
      The operation performs no I/O. It fails only when its input violates the
      value constraints described above or storage allocation cannot be completed.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.spl.SplFixedArray
visibility: public
modifiers:
  - static
---

[`SplFixedArray`](thp:std.spl.SplFixedArray)`::fromVector()` copies contiguous vector values into fixed slots.

## Behavior

Copies contiguous vector values into fixed slots.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = SplFixedArray::fromVector($values);
```

The call uses the signature and defaults documented above.

## See also

- [`SplFixedArray`](thp:std.spl.SplFixedArray)
