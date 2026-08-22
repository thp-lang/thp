---
kind: method
id: std.spl.SplFileObject::__construct
title: SplFileObject::__construct
summary:
  Opens the path with the requested stream mode and retains ownership of that
  stream for the object lifetime.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: filename
    type: string
    description: Path of the file to open.
  - name: mode
    type: string
    description: Mode selected from the values documented below.
    default: '"r"'
  - name: use_include_path
    type: bool
    description: Value supplied as $use_include_path.
    default: "false"
  - name: context
    type: mixed
    description: Optional stream context.
    default: "null"
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
owner: std.spl.SplFileObject
visibility: public
modifiers: []
---

[`SplFileObject`](thp:std.spl.SplFileObject)`::__construct()` opens the path with the requested stream mode and retains ownership of that stream for the object lifetime.

## Behavior

Opens the path with the requested stream mode and retains ownership of that stream for the object lifetime.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new SplFileObject($filename);
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
