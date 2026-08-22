---
kind: method
id: std.spl.SplFixedArray::setSize
title: SplFixedArray::setSize
summary: Resizes the storage.
name: setSize
order: 7
typeParameters: []
parameters:
  - name: size
    type: int
    description: Requested container or file size.
returns:
  type: "true"
  description: Returns true after the operation completes.
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

[`SplFixedArray`](thp:std.spl.SplFixedArray)`::setSize()` resizes the storage.

## Behavior

Resizes the storage.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$result = $instance->setSize($size);
```

The call uses the signature and defaults documented above.

## See also

- [`SplFixedArray`](thp:std.spl.SplFixedArray)
