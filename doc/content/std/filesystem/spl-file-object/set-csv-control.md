---
kind: method
id: std.spl.SplFileObject::setCsvControl
title: SplFileObject::setCsvControl
summary: Sets CSV delimiter, enclosure, and escape characters.
name: setCsvControl
order: 7
typeParameters: []
parameters:
  - name: separator
    type: string
    description: Single-character CSV field separator.
    default: '","'
  - name: enclosure
    type: string
    description: Single-character CSV enclosure delimiter.
    default: '"\""'
  - name: escape
    type: string
    description: CSV escape character; an empty string disables proprietary escaping.
    default: '""'
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

[`SplFileObject`](thp:std.spl.SplFileObject)`::setCsvControl()` sets CSV delimiter, enclosure, and escape characters.

## Behavior

Sets CSV delimiter, enclosure, and escape characters.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->setCsvControl();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
