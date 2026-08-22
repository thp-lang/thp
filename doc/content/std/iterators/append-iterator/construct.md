---
kind: method
id: std.spl.AppendIterator::__construct
title: AppendIterator::__construct
summary: Creates an empty sequence of iterators.
name: __construct
order: 1
typeParameters: []
parameters: []
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
owner: std.spl.AppendIterator
visibility: public
modifiers: []
---

[`AppendIterator`](thp:std.spl.AppendIterator)`::__construct()` creates an empty sequence of iterators.

## Behavior

Creates an empty sequence of iterators.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new AppendIterator();
```

The call uses the signature and defaults documented above.

## See also

- [`AppendIterator`](thp:std.spl.AppendIterator)
