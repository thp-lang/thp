---
kind: class
id: std.reflection.ReflectionUnionType
title: ReflectionUnionType
summary: Provides immutable VM-owned uniontype metadata.
name: ReflectionUnionType
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

`ReflectionUnionType` is an immutable descriptor produced from reflected declaration types.

## Methods

| Method                                                           | Description                                              |
| ---------------------------------------------------------------- | -------------------------------------------------------- |
| [`getTypes()`](thp:std.reflection.ReflectionUnionType::getTypes) | Returns normalized union members in deterministic order. |

Inherited metadata methods are declared by [`ReflectionType`](thp:std.reflection.ReflectionType).

## See also

- [Reflection](thp:std.reflection)
