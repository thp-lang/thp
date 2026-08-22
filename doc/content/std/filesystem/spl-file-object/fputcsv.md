---
kind: method
id: std.spl.SplFileObject::fputcsv
title: SplFileObject::fputcsv
summary: Writes one CSV record and returns its byte count.
name: fputcsv
order: 6
typeParameters: []
parameters:
  - name: fields
    type: vector<mixed>
    description: Fields written to the CSV record.
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
  - name: eol
    type: string
    description: Value supplied as $eol.
    default: '"\n"'
returns:
  type: int|false
  description: Writes one CSV record and returns its byte count.
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

[`SplFileObject`](thp:std.spl.SplFileObject)`::fputcsv()` writes one CSV record and returns its byte count.

## Behavior

Writes one CSV record and returns its byte count.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$result = $instance->fputcsv($fields);
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
