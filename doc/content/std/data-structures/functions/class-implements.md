---
kind: function
id: std.spl.class_implements
title: class_implements
summary: Returns the interfaces implemented by an object or type.
name: class_implements
order: 2
typeParameters: []
parameters:
  - name: object_or_class
    type: object|string
    description: Object instance or qualified type name to inspect.
returns:
  type: map<string, string>|false
  description:
    An insertion-ordered map of interface names keyed by those names, or false
    when the type cannot be resolved.
errors:
  - description:
      Lookup or iteration failures propagate. Concrete THP error classes for
      invalid inputs are not yet established.
related: []
status: experimental
availability: proposed
notice:
  This PHP-inspired function contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
module: data-structures
---

`class_implements()` returns the interfaces implemented by an object or type.

## Behavior

The result includes interfaces inherited through parent classes and parent interfaces.

## Example

```thp
$interfaces = class_implements(AppService::class);
```

## See also

- [SPL functions](thp:std.dataStructures)
- [PHP `class_implements()`](https://www.php.net/manual/en/function.class-implements.php)
