---
kind: module
id: std.reflection
title: Reflection
summary: Immutable descriptors for linked THP types, nominals, callables, properties, and parameters.
module: reflection
order: 75
status: experimental
availability: implemented
notice: >-
  The documented descriptor constructors and invokeArgs surface execute.
  Attributes, source inspection, closures, constants, and dynamic loading are
  not implemented.
---

Reflection inspects the verified linked program without loading source or
mutating accessibility. Construct class, function, method, property, and
parameter descriptors with their PHP-shaped `__construct()` APIs. Obtain type
descriptors from reflected declarations.

## Descriptors

- [`ReflectionType`](thp:std.reflection.ReflectionType),
  [`ReflectionNamedType`](thp:std.reflection.ReflectionNamedType), and
  [`ReflectionUnionType`](thp:std.reflection.ReflectionUnionType) describe
  semantic types.
- [`ReflectionClass`](thp:std.reflection.ReflectionClass) describes classes,
  interfaces, and traits.
- [`ReflectionFunction`](thp:std.reflection.ReflectionFunction) and
  [`ReflectionMethod`](thp:std.reflection.ReflectionMethod) share callable
  metadata from
  [`ReflectionFunctionAbstract`](thp:std.reflection.ReflectionFunctionAbstract).
- [`ReflectionProperty`](thp:std.reflection.ReflectionProperty) and
  [`ReflectionParameter`](thp:std.reflection.ReflectionParameter) retain exact
  member and signature metadata.
- [`ReflectionException`](thp:std.reflection.ReflectionException) reports
  invalid reflective operations.

See the [language reflection contract](thp:guide.languageReflection) for
discovery, ordering, invocation, construction, and PHP-difference semantics.
