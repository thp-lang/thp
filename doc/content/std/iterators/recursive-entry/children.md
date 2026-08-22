---
kind: method
id: std.spl.RecursiveEntry::children
title: RecursiveEntry::children
summary: Returns a cursor iterator for the entry's children, or null when it is a leaf.
name: children
order: 2
typeParameters: []
parameters: []
returns:
  type: ?RecursiveIterator<mixed, T>
  description: Returns a cursor iterator for the entry's children, or null when it is a leaf.
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
owner: std.spl.RecursiveEntry
visibility: public
modifiers: []
---

[`RecursiveEntry`](thp:std.spl.RecursiveEntry)`::children()` returns a cursor iterator for the entry's children, or null when it is a leaf.

## Behavior

Returns a cursor iterator for the entry's children, or null when it is a leaf.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->children();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveEntry`](thp:std.spl.RecursiveEntry)
