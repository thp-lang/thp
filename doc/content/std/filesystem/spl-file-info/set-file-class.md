---
kind: method
id: std.spl.SplFileInfo::setFileClass
title: SplFileInfo::setFileClass
summary: Selects the class returned by openFile().
name: setFileClass
order: 27
typeParameters: []
parameters:
  - name: class
    type: string
    description: Qualified class name used by this operation.
    default: '"SplFileObject"'
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

[`SplFileInfo`](thp:std.spl.SplFileInfo)`::setFileClass()` selects the class returned by openFile().

## Behavior

Selects the class returned by openFile().

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->setFileClass();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileInfo`](thp:std.spl.SplFileInfo)
