---
kind: guide
id: guide.languageOperators
title: Operators
summary: Summarizes THP arithmetic, comparison, logical, and access operators.
nav:
  section: language
  order: 70
status: experimental
availability: partial
notice: >-
  These operator contracts are proposed. The complete operator table and runtime behavior are not yet stable.
---

Operators combine, compare, convert, or access typed values.

## Operator groups

| Group           | Representative forms                           |
| --------------- | ---------------------------------------------- |
| Arithmetic      | `+`, `-`, `*`, `/`, `%`                        |
| Comparison      | `<`, `<=`, `>`, `>=`, `==`, `===`              |
| Logical         | `!`, `&&`, and the double-pipe logical OR form |
| Assignment      | `=`, compound assignment forms                 |
| Conditional     | `condition ? yes : no`, `??`                   |
| String          | `.`                                            |
| Access          | `$map[$key]`, `$object->property`              |
| Type conversion | Cast expressions                               |

## Precedence

Precedence determines how an unparenthesized expression groups. Use
parentheses when mixed operators would otherwise make the intent unclear.

```thp
$accepted = $enabled && ($count > 0);
$message = "count=" . $count;
```

Operands must satisfy the operator's type requirements. THP does not promise
all PHP coercions or PHP's complete operator set.

## String concatenation

`.` accepts operands statically typed as `string`, `int`, `float`, or `bool`
and returns `string`. Each operand uses the same canonical conversion as
`echo`, including `true` and `false` for booleans and deterministic
double-precision formatting for floats.

`null`, nullable unions, `mixed`, collections, and objects are rejected. Use
`??` or type narrowing before concatenating a nullable value:

```thp
$label = "owner=" . ($owner ?? "unassigned");
```

See [Basic syntax](thp:guide.languageBasicSyntax) for the complete output
conversion table.

## See also

- [Expressions](thp:guide.languageExpressions)
- [Types](thp:guide.languageTypes)
