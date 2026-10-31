---
kind: function
id: std.baseTypes.is_null
title: is_null
summary: Reports whether a value is null.
name: is_null
order: 4
typeParameters: []
parameters:
  - name: value
    type: mixed
    description: Value to inspect.
returns:
  type: bool
  description: True when value is null; otherwise false.
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

`is_null()` reports whether its argument is exactly `null`.

## Narrowing

When a local is passed directly to `is_null()` in a positive `if` or `elseif`
condition, its type is intersected with `null` inside that branch. An unrefined
`mixed` local therefore becomes `null`. Its previous type is restored after the
branch.

Negation, boolean composition, loops, early exits, and branch joins do not
produce this refinement.

## Example

```thp
function missing(mixed $value): bool {
    if (is_null($value)) {
        return true;
    }

    return false;
}
```

## See also

- [Runtime type guards](thp:guide.languageTypes)
