---
kind: function
id: std.baseTypes.is_vector
title: is_vector
summary: Reports whether a value is a vector.
name: is_vector
order: 6
typeParameters: []
parameters:
  - name: value
    type: mixed
    description: Value to inspect without conversion.
returns:
  type: bool
  description: True when value is a vector; otherwise false.
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

`is_vector()` reports whether its argument is a vector, regardless of the
vector's runtime element descriptor. It performs no conversion.

## Narrowing

When a local is passed directly to `is_vector()` in a positive `if` or
`elseif` condition, its type is intersected with `vector<mixed>` inside that
branch. An unrefined `mixed` local therefore becomes `vector<mixed>`; retrieved
elements remain `mixed`. The local's previous type is restored after the
branch.

Negation, boolean composition, loops, early exits, and branch joins do not
produce this refinement.

## Example

```thp
function has_values(mixed $value): bool {
    if (is_vector($value)) {
        return count($value) > 0;
    }

    return false;
}
```

## See also

- [`is_map()`](thp:std.baseTypes.is_map)
- [Runtime type guards](thp:guide.languageTypes)
