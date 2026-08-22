---
kind: guide
id: guide.languageExpressions
title: Expressions
summary: Describes THP syntax that evaluates to values.
nav:
  section: language
  order: 60
status: experimental
availability: partial
notice: >-
  Literals, variables, typed operators, statically resolved calls, object
  construction, native collection access and assignment, and `match` execute.
  Ternaries, dynamic calls, and general reference expressions remain proposals.
---

An expression produces a value or performs an operation. Literals, variables,
operator applications, calls, object construction, collection access, and
`match` are PHP-shaped expression forms in THP.

```thp
$subtotal = $price * $quantity;
$label = $subtotal > 100 ? "large" : "standard";
```

## Evaluation

Operands are evaluated according to operator precedence. Parentheses make the
intended grouping explicit.

```thp
$result = ($left + $right) * $scale;
```

Operands and call arguments evaluate once from left to right. Named arguments
retain their source evaluation order even when binding to parameters in a
different order. `match` evaluates its subject once, then conditions lazily
from left to right.

## Typed results

Each executable expression has a type. Assignments, arguments, returns,
properties, and collection elements are checked against their expected types.

Dynamic calls, capturing closures, and general reference expressions do not yet
have stable THP contracts.

## Collection-element assignment

A variable-rooted vector or map path may be assigned:

```thp
$matrix: vector<vector<int>> = [[1, 2], [3, 4]];
$matrix[0][1] = 9;

$scores: map<string, int> = {"Ada" => 10};
$scores["Grace"] = 12;
```

Every index and intermediate collection is checked statically. Vector
assignment replaces an existing non-negative integer offset; it does not grow
the vector. Map assignment replaces an existing key without changing its
position or appends a new key/value pair at the end.

Collections have value semantics. An assignment detaches shared storage before
mutation, and nested assignment rebuilds the changed path from the inside out.
The root variable receives the rebuilt collection only after all index
expressions and the right-hand side have evaluated successfully.

Property-rooted and arbitrary-expression-rooted paths are not assignment
targets in the current implementation.

## Match expressions

See [Control structures](thp:guide.languageControlStructures) for `match`
ordering, typing, and unmatched-subject behavior.

## See also

- [Types](thp:guide.languageTypes)
- [Operators](thp:guide.languageOperators)
- [Functions](thp:guide.languageFunctions)
- [Control structures](thp:guide.languageControlStructures)
