---
kind: method
id: std.spl.SplFileObject::setMaxLineLen
title: SplFileObject::setMaxLineLen
summary: Sets the maximum line length; zero removes the limit.
name: setMaxLineLen
order: 21
typeParameters: []
parameters:
  - name: max_length
    type: int
    description: Maximum line length; 0 removes the limit.
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

[`SplFileObject`](thp:std.spl.SplFileObject)`::setMaxLineLen()` sets the maximum line length; zero removes the limit.

## Behavior

Sets the maximum line length; zero removes the limit.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->setMaxLineLen($max_length);
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
