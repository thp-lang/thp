---
kind: function
id: std.spl.class_parents
title: class_parents
summary: Returns the parent classes of an object or type.
name: class_parents
order: 3
typeParameters: []
parameters:
  - name: object_or_class
    type: object|string
    description: Object instance or qualified type name to inspect.
  - name: autoload
    type: bool
    description: Whether an unknown type name may trigger autoloading.
    default: "true"
returns:
  type: map<string, string>|false
  description:
    An insertion-ordered map of parent class names keyed by those names, or
    false when the type cannot be resolved.
errors:
  - description:
      Lookup, callback, or iteration failures propagate. Concrete THP error
      classes for invalid inputs are not yet established.
related: []
status: experimental
availability: proposed
notice:
  This PHP-inspired function contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
module: data-structures
---

`class_parents()` returns the parent classes of an object or type.

## Behavior

The result walks the complete parent chain and does not include the inspected class itself.

## Example

```thp
$parents = class_parents(AdminController::class);
```

## See also

- [SPL functions](thp:std.dataStructures)
- [PHP `class_parents()`](https://www.php.net/manual/en/function.class-parents.php)
