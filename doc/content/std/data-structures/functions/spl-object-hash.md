---
kind: function
id: std.spl.spl_object_hash
title: spl_object_hash
summary: Returns a process-local identity string for an object.
name: spl_object_hash
order: 15
typeParameters: []
parameters:
  - name: object
    type: object
    description: Object whose identity is requested.
returns:
  type: string
  description: An opaque string that identifies the object while that object remains alive.
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

`spl_object_hash()` returns a process-local identity string for an object.

## Behavior

Equal state does not imply equal identity. The value is not stable across processes or object lifetimes and must not be persisted as a durable identifier.

## Example

```thp
$identity = spl_object_hash($connection);
```

## See also

- [SPL functions](thp:std.dataStructures)
- [PHP `spl_object_hash()`](https://www.php.net/manual/en/function.spl-object-hash.php)
