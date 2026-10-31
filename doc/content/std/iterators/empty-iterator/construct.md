---
kind: method
id: std.spl.EmptyIterator::__construct
title: EmptyIterator::__construct
summary: Creates an iterator that is already exhausted.
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
availability: implemented
notice: This experimental contract is implemented in the standalone compiler and VM.
version: "0.5"
owner: std.spl.EmptyIterator
visibility: public
modifiers: []
---

[`EmptyIterator`](thp:std.spl.EmptyIterator)`::__construct()` creates an iterator that is already exhausted.

## Behavior

Creates an iterator that is already exhausted.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new EmptyIterator<int, string>();
```

The call uses the signature and defaults documented above.

## See also

- [`EmptyIterator`](thp:std.spl.EmptyIterator)
