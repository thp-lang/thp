---
kind: class
id: std.reflection.ReflectionParameter
title: ReflectionParameter
summary: Provides immutable VM-owned parameter metadata.
name: ReflectionParameter
module: reflection
typeParameters: []
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
notice: This VM-owned reflection descriptor and its constructor are implemented; closure and invokable-object inputs remain unsupported.
version: "0.1"
---

`ReflectionParameter` reflects a parameter by zero-based position or exact name. Its callable is a function name or an exact two-value `[object|string, string]` method vector.

## Methods

| Method                                                                                         | Description                                                            |
| ---------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| [`__construct()`](thp:std.reflection.ReflectionParameter::__construct)                         | Reflects a parameter of a function or method callable.                 |
| [`getName()`](thp:std.reflection.ReflectionParameter::getName)                                 | Returns the case-sensitive parameter name.                             |
| [`getPosition()`](thp:std.reflection.ReflectionParameter::getPosition)                         | Returns the zero-based parameter position.                             |
| [`getType()`](thp:std.reflection.ReflectionParameter::getType)                                 | Returns the declared parameter type with generic substitution.         |
| [`getDeclaringFunction()`](thp:std.reflection.ReflectionParameter::getDeclaringFunction)       | Returns the function or method descriptor that declared the parameter. |
| [`isDefaultValueAvailable()`](thp:std.reflection.ReflectionParameter::isDefaultValueAvailable) | Reports whether a constant default is retained.                        |
| [`getDefaultValue()`](thp:std.reflection.ReflectionParameter::getDefaultValue)                 | Returns a fresh materialization of the retained default.               |
| [`isOptional()`](thp:std.reflection.ReflectionParameter::isOptional)                           | Reports whether the parameter has a default or is variadic.            |
| [`isVariadic()`](thp:std.reflection.ReflectionParameter::isVariadic)                           | Reports whether the parameter captures remaining positional arguments. |

## See also

- [Reflection](thp:std.reflection)
