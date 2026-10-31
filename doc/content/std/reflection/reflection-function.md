---
kind: class
id: std.reflection.ReflectionFunction
title: ReflectionFunction
summary: Provides immutable VM-owned function metadata.
name: ReflectionFunction
module: reflection
typeParameters: []
parent:
  id: std.reflection.ReflectionFunctionAbstract
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
notice: This VM-owned reflection descriptor and its constructor are implemented; closure inputs remain unsupported.
version: "0.1"
---

`ReflectionFunction` reflects a case-sensitive linked top-level function name.

## Methods

| Method                                                                | Description                                                            |
| --------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| [`__construct()`](thp:std.reflection.ReflectionFunction::__construct) | Reflects a linked top-level function by canonical name.                |
| [`invokeArgs()`](thp:std.reflection.ReflectionFunction::invokeArgs)   | Invokes the exact function with checked positional or named arguments. |

Inherited metadata methods are declared by [`ReflectionFunctionAbstract`](thp:std.reflection.ReflectionFunctionAbstract).

## See also

- [Reflection](thp:std.reflection)
