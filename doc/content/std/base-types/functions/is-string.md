---
kind: function
id: std.baseTypes.is_string
title: is_string
summary: Reports whether a value is a byte string.
name: is_string
order: 1
typeParameters: []
parameters:
  - name: value
    type: mixed
    description: Value to inspect without conversion.
returns:
  type: bool
  description: True when value is a byte string; otherwise false.
errors: []
related: []
status: experimental
availability: implemented
notice: >-
  The compiler and reference VM implement this predicate and its direct-positive
  branch narrowing.
version: "0.1"
module: base-types
---

`is_string()` reports whether its argument is a `string`. THP strings are byte
strings, so valid UTF-8 is not required. The function inspects the value without
converting it.

## Narrowing

When a local is passed directly to `is_string()` in a positive `if` or
`elseif` condition, its type is intersected with `string` inside that branch.
An unrefined `mixed` local therefore becomes `string`. Its previous type is
restored after the branch.

Negation, boolean composition, loops, early exits, and branch joins do not
produce this refinement.

## Example

```thp
function display(mixed $value): string {
    if (is_string($value)) {
        return $value;
    }

    return "not a string";
}
```

## See also

- [Runtime type guards](thp:guide.languageTypes)
