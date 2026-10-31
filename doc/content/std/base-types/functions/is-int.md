---
kind: function
id: std.baseTypes.is_int
title: is_int
summary: Reports whether a value is an integer.
name: is_int
order: 2
typeParameters: []
parameters:
  - name: value
    type: mixed
    description: Value to inspect without conversion.
returns:
  type: bool
  description: True when value is an integer; otherwise false.
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

`is_int()` reports whether its argument is an `int`. Numeric strings and
floating-point values return `false`; the function performs no conversion.

## Narrowing

When a local is passed directly to `is_int()` in a positive `if` or `elseif`
condition, its type is intersected with `int` inside that branch. An unrefined
`mixed` local therefore becomes `int`. Its previous type is restored after the
branch.

Negation, boolean composition, loops, early exits, and branch joins do not
produce this refinement.

## Example

```thp
function increment(mixed $value): int {
    if (is_int($value)) {
        return $value + 1;
    }

    return 0;
}
```

## See also

- [`is_numeric()`](thp:std.baseTypes.is_numeric)
- [Runtime type guards](thp:guide.languageTypes)
