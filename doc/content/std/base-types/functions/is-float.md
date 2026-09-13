---
kind: function
id: std.baseTypes.is_float
title: is_float
summary: Reports whether a value is a floating-point number.
name: is_float
order: 3
typeParameters: []
parameters:
  - name: value
    type: mixed
    description: Value to inspect without conversion.
returns:
  type: bool
  description: True when value is a floating-point number; otherwise false.
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

`is_float()` reports whether its argument is a `float`. Integers and numeric
strings return `false`; the function performs no conversion.

## Narrowing

When a local is passed directly to `is_float()` in a positive `if` or `elseif`
condition, its type is intersected with `float` inside that branch. An
unrefined `mixed` local therefore becomes `float`. Its previous type is restored
after the branch.

Negation, boolean composition, loops, early exits, and branch joins do not
produce this refinement.

## Example

```thp
function halve(mixed $value): float {
    if (is_float($value)) {
        return $value / 2.0;
    }

    return 0.0;
}
```

## See also

- [`is_numeric()`](thp:std.baseTypes.is_numeric)
- [Runtime type guards](thp:guide.languageTypes)
