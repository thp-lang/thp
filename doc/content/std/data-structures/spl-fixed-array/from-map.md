---
kind: method
id: std.spl.SplFixedArray::fromMap
title: SplFixedArray::fromMap
summary: Copies integer-keyed values, preserving positions.
name: fromMap
order: 5
typeParameters: []
parameters:
  - name: values
    type: map<int, T>
    description: Initial values consumed or stored by the operation.
returns:
  type: SplFixedArray<T>
  description: Copies integer-keyed values, preserving positions.
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

[`SplFixedArray`](thp:std.spl.SplFixedArray)`::fromMap()` copies integer-keyed values, preserving positions.

## Behavior

Copies integer-keyed values, preserving positions.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = SplFixedArray::fromMap($values);
```

The call uses the signature and defaults documented above.

## See also

- [`SplFixedArray`](thp:std.spl.SplFixedArray)
