---
kind: function
id: std.spl.spl_object_id
title: spl_object_id
summary: Returns a process-local integer identity for an object.
name: spl_object_id
order: 16
typeParameters: []
parameters:
  - name: object
    type: object
    description: Object whose identity is requested.
returns:
  type: int
  description: An integer that distinguishes the object from other live objects.
errors:
  - description:
      The static object parameter rejects non-object values before the function
      runs. No other normal failure is expected.
related: []
status: experimental
availability: proposed
notice:
  This PHP-inspired function contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
module: data-structures
---

`spl_object_id()` returns a process-local integer identity for an object.

## Behavior

The identifier may be reused after an object is destroyed. It is suitable only for transient in-process identity comparisons.

## Example

```thp
$identity = spl_object_id($connection);
```

## See also

- [SPL functions](thp:std.dataStructures)
- [PHP `spl_object_id()`](https://www.php.net/manual/en/function.spl-object-id.php)
