---
kind: class
id: std.reflection.ReflectionFunctionAbstract
title: ReflectionFunctionAbstract
summary: Provides immutable VM-owned functionabstract metadata.
name: ReflectionFunctionAbstract
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

`ReflectionFunctionAbstract` is the abstract base for VM-initialized function and method descriptors.

## Methods

| Method                                                                                                            | Description                                                 |
| ----------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------- |
| [`getName()`](thp:std.reflection.ReflectionFunctionAbstract::getName)                                             | Returns the canonical callable name.                        |
| [`getShortName()`](thp:std.reflection.ReflectionFunctionAbstract::getShortName)                                   | Returns the unqualified callable name.                      |
| [`getNamespaceName()`](thp:std.reflection.ReflectionFunctionAbstract::getNamespaceName)                           | Returns the callable namespace without a leading separator. |
| [`getModuleName()`](thp:std.reflection.ReflectionFunctionAbstract::getModuleName)                                 | Returns the canonical owning module name.                   |
| [`getNumberOfParameters()`](thp:std.reflection.ReflectionFunctionAbstract::getNumberOfParameters)                 | Returns the total parameter count.                          |
| [`getNumberOfRequiredParameters()`](thp:std.reflection.ReflectionFunctionAbstract::getNumberOfRequiredParameters) | Returns the count before optional and variadic parameters.  |
| [`getParameters()`](thp:std.reflection.ReflectionFunctionAbstract::getParameters)                                 | Returns parameter descriptors in declaration order.         |
| [`getReturnType()`](thp:std.reflection.ReflectionFunctionAbstract::getReturnType)                                 | Returns the declared return type.                           |
| [`isVariadic()`](thp:std.reflection.ReflectionFunctionAbstract::isVariadic)                                       | Reports whether the callable has a variadic parameter.      |
| [`isInternal()`](thp:std.reflection.ReflectionFunctionAbstract::isInternal)                                       | Reports whether the callable is runtime-owned.              |
| [`isUserDefined()`](thp:std.reflection.ReflectionFunctionAbstract::isUserDefined)                                 | Reports whether the callable came from linked user source.  |

## See also

- [Reflection](thp:std.reflection)
