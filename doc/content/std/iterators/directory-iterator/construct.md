---
kind: method
id: std.spl.DirectoryIterator::__construct
title: DirectoryIterator::__construct
summary: Opens $directory and prepares a cursor iterator over its entries.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: directory
    type: string
    description: Directory to open.
returns:
  type: void
  description: This callable does not return a value.
errors:
  - description:
      Construction fails when an argument violates the documented contract or an
      underlying resource cannot be created. Concrete THP error classes remain
      experimental unless named above.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.spl.DirectoryIterator
visibility: public
modifiers: []
---

[`DirectoryIterator`](thp:std.spl.DirectoryIterator)`::__construct()` opens $directory and prepares a cursor iterator over its entries.

## Behavior

Opens $directory and prepares a cursor iterator over its entries.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new DirectoryIterator($directory);
```

The call uses the signature and defaults documented above.

## See also

- [`DirectoryIterator`](thp:std.spl.DirectoryIterator)
