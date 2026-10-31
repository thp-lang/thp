---
kind: class
id: std.reflection.ReflectionType
title: ReflectionType
summary: Provides immutable VM-owned type metadata.
name: ReflectionType
module: reflection
typeParameters: []
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
notice: This abstract VM-owned reflection base class is implemented.
version: "0.1"
---

`ReflectionType` is the abstract base for VM-produced declaration type descriptors.

## Methods

| Method                                                                      | Description                                                           |
| --------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| [`allowsNull()`](thp:std.reflection.ReflectionType::allowsNull)             | Reports whether the reflected type accepts null.                      |
| [`getDisplayName()`](thp:std.reflection.ReflectionType::getDisplayName)     | Returns the normalized semantic type spelling.                        |
| [`equals()`](thp:std.reflection.ReflectionType::equals)                     | Compares two reflected types semantically.                            |
| [`isAssignableFrom()`](thp:std.reflection.ReflectionType::isAssignableFrom) | Reports whether values of the other type are assignable to this type. |

## See also

- [Reflection](thp:std.reflection)
