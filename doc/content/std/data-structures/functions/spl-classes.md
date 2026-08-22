---
kind: function
id: std.spl.spl_classes
title: spl_classes
summary: Returns the SPL classes and interfaces available to THP.
name: spl_classes
order: 14
typeParameters: []
parameters: []
returns:
  type: vector<string>
  description: A snapshot of available SPL type names.
errors:
  - description: The function is not expected to fail during normal operation.
related: []
status: experimental
availability: proposed
notice:
  This PHP-inspired function contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
module: data-structures
---

`spl_classes()` returns the SPL classes and interfaces available to THP.

## Behavior

The returned snapshot contains only SPL classes and interfaces implemented by the active runtime, sorted by ascending qualified name.

## Example

```thp
foreach (spl_classes() as $class) {
    print($class);
}
```

## See also

- [SPL functions](thp:std.dataStructures)
- [PHP `spl_classes()`](https://www.php.net/manual/en/function.spl-classes.php)
