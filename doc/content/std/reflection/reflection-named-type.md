---
kind: class
id: std.reflection.ReflectionNamedType
title: ReflectionNamedType
summary: Provides immutable VM-owned namedtype metadata.
name: ReflectionNamedType
module: reflection
typeParameters: []
parent:
  id: std.reflection.ReflectionType
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
notice: This VM-produced reflection descriptor is implemented; direct construction is unavailable.
version: "0.1"
---

`ReflectionNamedType` is an immutable descriptor produced from reflected declaration and class types.

## Methods

| Method                                                                           | Description                                                            |
| -------------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| [`getName()`](thp:std.reflection.ReflectionNamedType::getName)                   | Returns the scalar, collection, nominal, or type-parameter name.       |
| [`isBuiltin()`](thp:std.reflection.ReflectionNamedType::isBuiltin)               | Reports whether the name denotes a built-in scalar or collection type. |
| [`isTypeParameter()`](thp:std.reflection.ReflectionNamedType::isTypeParameter)   | Reports whether the descriptor denotes a declaration type parameter.   |
| [`getTypeArguments()`](thp:std.reflection.ReflectionNamedType::getTypeArguments) | Returns concrete generic arguments in declaration order.               |

Inherited metadata methods are declared by [`ReflectionType`](thp:std.reflection.ReflectionType).

## See also

- [Reflection](thp:std.reflection)
