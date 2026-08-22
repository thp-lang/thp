---
kind: method
id: std.spl.SplFileInfo::setInfoClass
title: SplFileInfo::setInfoClass
summary: Selects the class returned by information methods.
name: setInfoClass
order: 28
typeParameters: []
parameters:
  - name: class
    type: string
    description: Qualified class name used by this operation.
    default: '"SplFileInfo"'
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
owner: std.spl.SplFileInfo
visibility: public
modifiers: []
---

[`SplFileInfo`](thp:std.spl.SplFileInfo)`::setInfoClass()` selects the class returned by information methods.

## Behavior

Selects the class returned by information methods.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->setInfoClass();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileInfo`](thp:std.spl.SplFileInfo)
