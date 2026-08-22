---
kind: method
id: std.spl.SplFileObject::fgetcsv
title: SplFileObject::fgetcsv
summary: Reads and parses the next CSV record.
name: fgetcsv
order: 5
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
  type: vector<?string>|false
  description: Reads and parses the next CSV record.
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

[`SplFileObject`](thp:std.spl.SplFileObject)`::fgetcsv()` reads and parses the next CSV record.

## Behavior

Reads and parses the next CSV record.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->fgetcsv();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
