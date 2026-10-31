---
kind: function
id: std.baseTypes.is_numeric
title: is_numeric
summary: Reports whether a value is an integer, float, or decimal numeric string.
name: is_numeric
order: 5
typeParameters: []
parameters:
  - name: value
    type: mixed
    description: Value to inspect without conversion.
returns:
  type: bool
  description: True for an integer, float, or accepted numeric string; otherwise false.
errors: []
related: []
status: experimental
availability: implemented
notice: >-
  The compiler and reference VM implement this predicate, numeric-string grammar,
  and direct-positive branch narrowing.
version: "0.1"
module: base-types
---

`is_numeric()` returns `true` for every `int` and `float`, and for a string that
matches THP's decimal numeric-string grammar. It inspects the value without
converting it.

## Numeric strings

A numeric string permits optional ASCII whitespace before and after the value,
an optional sign, and a decimal integer, fraction, or exponent form. At least
one decimal digit is required. Examples include `"42"`, `" -1.5 "`, `".5"`,
`"1."`, and `"6.02e23"`.

Empty and whitespace-only strings, partial numbers, hexadecimal or binary
prefixes, underscores, `INF`, `NAN`, non-ASCII bytes, and trailing
non-whitespace text are rejected.

## Narrowing

When a local is passed directly to `is_numeric()` in a positive `if` or
`elseif` condition, its type is intersected with `int|float|string` inside that
branch. An unrefined `mixed` local therefore becomes the complete union. A
numeric string remains a `string`; the guard never converts it. The local's
previous type is restored after the branch.

Negation, boolean composition, loops, early exits, and branch joins do not
produce this refinement.

## Example

```thp
function numeric(mixed $value): bool {
    if (is_numeric($value)) {
        return true;
    }

    return false;
}
```

## See also

- [`is_int()`](thp:std.baseTypes.is_int)
- [`is_float()`](thp:std.baseTypes.is_float)
- [Runtime type guards](thp:guide.languageTypes)
- [PHP numeric-string grammar](https://www.php.net/manual/en/language.types.numeric-strings.php)
