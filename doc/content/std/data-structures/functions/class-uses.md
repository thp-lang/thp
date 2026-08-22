---
kind: function
id: std.spl.class_uses
title: class_uses
summary: Returns the traits used directly by an object or type.
name: class_uses
order: 4
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
    An insertion-ordered map of trait names keyed by those names, or false when
    the type cannot be resolved.
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

`class_uses()` returns the traits used directly by an object or type.

## Behavior

Following PHP, the result covers traits used by the inspected class and does not recursively merge traits from parent classes.

## Example

```thp
$traits = class_uses(AuditedRecord::class);
```

## See also

- [SPL functions](thp:std.dataStructures)
- [PHP `class_uses()`](https://www.php.net/manual/en/function.class-uses.php)
