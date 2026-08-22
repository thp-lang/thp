---
kind: method
id: std.spl.SplFixedArray::__construct
title: SplFixedArray::__construct
summary:
  Allocates $size zero-based slots. New slots initially contain null, so the
  final generic nullability model remains unsettled.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: size
    type: int
    description: Requested container or file size.
    default: "0"
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
owner: std.spl.SplFixedArray
visibility: public
modifiers: []
---

[`SplFixedArray`](thp:std.spl.SplFixedArray)`::__construct()` allocates $size zero-based slots. New slots initially contain null, so the final generic nullability model remains unsettled.

## Behavior

Allocates $size zero-based slots. New slots initially contain null, so the final generic nullability model remains unsettled.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new SplFixedArray();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFixedArray`](thp:std.spl.SplFixedArray)
