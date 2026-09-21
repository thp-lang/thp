---
kind: class
id: std.reflection.ReflectionProperty
title: ReflectionProperty
summary: Provides immutable VM-owned property metadata.
name: ReflectionProperty
module: reflection
typeParameters: []
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
notice: This VM-owned reflection descriptor and its constructor are implemented.
version: "0.1"
---

`ReflectionProperty` reflects a property on an object or linked nominal name. Object inputs preserve concrete generic arguments.

## Methods

| Method                                                                            | Description                                                            |
| --------------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| [`__construct()`](thp:std.reflection.ReflectionProperty::__construct)             | Reflects a named property on an object or linked nominal.              |
| [`getName()`](thp:std.reflection.ReflectionProperty::getName)                     | Returns the declared property name.                                    |
| [`getDeclaringClass()`](thp:std.reflection.ReflectionProperty::getDeclaringClass) | Returns the consuming or declaring class.                              |
| [`getOriginTrait()`](thp:std.reflection.ReflectionProperty::getOriginTrait)       | Returns the ultimate trait descriptor, if composed from a trait.       |
| [`getType()`](thp:std.reflection.ReflectionProperty::getType)                     | Returns the declared property type with concrete generic substitution. |
| [`hasDefaultValue()`](thp:std.reflection.ReflectionProperty::hasDefaultValue)     | Reports whether a constant default is retained.                        |
| [`getDefaultValue()`](thp:std.reflection.ReflectionProperty::getDefaultValue)     | Returns a fresh retained default, or null when none exists.            |
| [`isPublic()`](thp:std.reflection.ReflectionProperty::isPublic)                   | Reports whether the declaration is public.                             |
| [`isProtected()`](thp:std.reflection.ReflectionProperty::isProtected)             | Reports whether the declaration is protected.                          |
| [`isPrivate()`](thp:std.reflection.ReflectionProperty::isPrivate)                 | Reports whether the declaration is private.                            |
| [`isStatic()`](thp:std.reflection.ReflectionProperty::isStatic)                   | Reports whether the declaration is static.                             |
| [`getValue()`](thp:std.reflection.ReflectionProperty::getValue)                   | Reads the exact retained property slot from a compatible object.       |
| [`setValue()`](thp:std.reflection.ReflectionProperty::setValue)                   | Writes a checked value to the exact retained property slot.            |

## See also

- [Reflection](thp:std.reflection)
