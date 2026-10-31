---
kind: function
id: std.baseTypes.is_map
title: is_map
summary: Reports whether a value is a map.
name: is_map
order: 7
typeParameters: []
parameters:
  - name: value
    type: mixed
    description: Value to inspect without conversion.
returns:
  type: bool
  description: True when value is a map; otherwise false.
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

`is_map()` reports whether its argument is a map, regardless of the map's
runtime key and value descriptors. It performs no conversion.

## Narrowing

When a local is passed directly to `is_map()` in a positive `if` or `elseif`
condition, its type is intersected with `map<mixed, mixed>` inside that branch.
An unrefined `mixed` local therefore becomes `map<mixed, mixed>`; retrieved keys
and values remain `mixed`. The local's previous type is restored after the
branch.

Negation, boolean composition, loops, early exits, and branch joins do not
produce this refinement.

## Example

```thp
function has_entries(mixed $value): bool {
    if (is_map($value)) {
        return count($value) > 0;
    }

    return false;
}
```

## See also

- [`is_vector()`](thp:std.baseTypes.is_vector)
- [Runtime type guards](thp:guide.languageTypes)
