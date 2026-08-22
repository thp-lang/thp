---
kind: method
id: std.spl.SplFileObject::csvRows
title: SplFileObject::csvRows
summary: Returns a typed CSV-row iterator.
name: csvRows
order: 26
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
  type: Iterator<int, vector<?string>>
  description: Returns a typed CSV-row iterator.
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

[`SplFileObject`](thp:std.spl.SplFileObject)`::csvRows()` returns a typed CSV-row iterator.

## Behavior

Returns a typed CSV-row iterator.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->csvRows();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
