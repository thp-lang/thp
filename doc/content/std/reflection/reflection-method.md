---
kind: class
id: std.reflection.ReflectionMethod
title: ReflectionMethod
summary: Provides immutable VM-owned method metadata.
name: ReflectionMethod
module: reflection
typeParameters: []
parent:
  id: std.reflection.ReflectionFunctionAbstract
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
notice: This VM-owned reflection descriptor and its two-argument constructor are implemented.
version: "0.1"
---

`ReflectionMethod` reflects a method on an object or linked nominal name. Object inputs preserve concrete generic arguments.

## Methods

| Method                                                                          | Description                                                              |
| ------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| [`__construct()`](thp:std.reflection.ReflectionMethod::__construct)             | Reflects a named method on an object or linked nominal.                  |
| [`getDeclaringClass()`](thp:std.reflection.ReflectionMethod::getDeclaringClass) | Returns the consuming or declaring class.                                |
| [`getOriginTrait()`](thp:std.reflection.ReflectionMethod::getOriginTrait)       | Returns the ultimate trait descriptor, if composed from a trait.         |
| [`getOriginMethod()`](thp:std.reflection.ReflectionMethod::getOriginMethod)     | Returns the ultimate trait method descriptor, if present.                |
| [`isPublic()`](thp:std.reflection.ReflectionMethod::isPublic)                   | Reports whether the declaration is public.                               |
| [`isProtected()`](thp:std.reflection.ReflectionMethod::isProtected)             | Reports whether the declaration is protected.                            |
| [`isPrivate()`](thp:std.reflection.ReflectionMethod::isPrivate)                 | Reports whether the declaration is private.                              |
| [`isStatic()`](thp:std.reflection.ReflectionMethod::isStatic)                   | Reports whether the declaration is static.                               |
| [`isAbstract()`](thp:std.reflection.ReflectionMethod::isAbstract)               | Reports whether the declaration is abstract.                             |
| [`isFinal()`](thp:std.reflection.ReflectionMethod::isFinal)                     | Reports whether the declaration is final.                                |
| [`isConstructor()`](thp:std.reflection.ReflectionMethod::isConstructor)         | Reports whether this is the __construct declaration.                     |
| [`invokeArgs()`](thp:std.reflection.ReflectionMethod::invokeArgs)               | Invokes the exact retained method with a checked receiver and arguments. |

Inherited metadata methods are declared by [`ReflectionFunctionAbstract`](thp:std.reflection.ReflectionFunctionAbstract).

## See also

- [Reflection](thp:std.reflection)
