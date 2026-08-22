---
kind: method
id: std.spl.SplFileObject::setFlags
title: SplFileObject::setFlags
summary: Sets line-iteration flags.
name: setFlags
order: 19
typeParameters: []
parameters:
  - name: flags
    type: int
    description: Bit mask selecting the documented options.
returns:
  type: void
  description: This method does not return a value.
errors:
  - description:
      Underlying I/O failures follow the return sentinel shown in the signature
      or propagate as the experimental THP I/O failure where no sentinel is available.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.spl.SplFileObject
visibility: public
modifiers: []
---

[`SplFileObject`](thp:std.spl.SplFileObject)`::setFlags()` sets line-iteration flags.

## Behavior

Sets line-iteration flags.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->setFlags($flags);
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
