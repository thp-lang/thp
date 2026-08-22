---
kind: guide
id: guide.languageVariables
title: Variables
summary: Describes THP variable bindings, inference, and explicit declarations.
nav:
  section: language
  order: 40
status: experimental
availability: partial
notice: >-
  These binding and scope rules are intended language contracts and are not an implementation-status claim.
---

Variable names begin with `$`. A binding can infer its type from an initializer
or declare the type explicitly.

```thp
$total = 0;
$count: int = 0;
```

## Assignment

Assignment stores a value in an existing binding. The value must satisfy the
binding's inferred or declared type.

```thp
$attempts: int = 1;
$attempts = $attempts + 1;
```

Collection indexing and object properties are assignable when the collection or
property permits mutation.

## Scope

Parameters and variables declared inside a function are local to that call.
Class methods access the receiver with `$this`.

Variable variables and general reference aliasing are not part of the intended
THP contract.

## Predefined bindings

THP does not currently promise PHP's superglobals. See
[Predefined variables](thp:guide.languagePredefinedVariables) for the established status of
runtime-provided bindings.

## See also

- [Types](thp:guide.languageTypes)
- [Expressions](thp:guide.languageExpressions)
- [References](thp:guide.languageReferences)
