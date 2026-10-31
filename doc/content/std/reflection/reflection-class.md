---
kind: class
id: std.reflection.ReflectionClass
title: ReflectionClass
summary: Provides immutable VM-owned class metadata.
name: ReflectionClass
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

`ReflectionClass` accepts an object or canonical class, interface, or trait name. Object construction preserves concrete generic arguments; string lookup erases them.

## Methods

| Method                                                                                 | Description                                                               |
| -------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| [`__construct()`](thp:std.reflection.ReflectionClass::__construct)                     | Reflects an object or linked class, interface, or trait name.             |
| [`getName()`](thp:std.reflection.ReflectionClass::getName)                             | Returns the canonical qualified nominal name.                             |
| [`getShortName()`](thp:std.reflection.ReflectionClass::getShortName)                   | Returns the unqualified nominal name.                                     |
| [`getNamespaceName()`](thp:std.reflection.ReflectionClass::getNamespaceName)           | Returns the nominal namespace without a leading separator.                |
| [`getModuleName()`](thp:std.reflection.ReflectionClass::getModuleName)                 | Returns the canonical owning module name.                                 |
| [`getType()`](thp:std.reflection.ReflectionClass::getType)                             | Returns the concrete nominal type when generic arguments are known.       |
| [`isAbstract()`](thp:std.reflection.ReflectionClass::isAbstract)                       | Reports whether the nominal is abstract.                                  |
| [`isFinal()`](thp:std.reflection.ReflectionClass::isFinal)                             | Reports whether the nominal is final.                                     |
| [`isInterface()`](thp:std.reflection.ReflectionClass::isInterface)                     | Reports whether the nominal is an interface.                              |
| [`isTrait()`](thp:std.reflection.ReflectionClass::isTrait)                             | Reports whether the nominal is a trait.                                   |
| [`isInternal()`](thp:std.reflection.ReflectionClass::isInternal)                       | Reports whether the nominal is owned by the runtime.                      |
| [`isUserDefined()`](thp:std.reflection.ReflectionClass::isUserDefined)                 | Reports whether the nominal came from linked user source.                 |
| [`isInstantiable()`](thp:std.reflection.ReflectionClass::isInstantiable)               | Reports whether reflective construction is available.                     |
| [`getParentClass()`](thp:std.reflection.ReflectionClass::getParentClass)               | Returns the instantiated direct parent descriptor, if any.                |
| [`getInterfaces()`](thp:std.reflection.ReflectionClass::getInterfaces)                 | Returns direct and retained interface descriptors in deterministic order. |
| [`getTraits()`](thp:std.reflection.ReflectionClass::getTraits)                         | Returns directly composed trait descriptors in source order.              |
| [`getConstructor()`](thp:std.reflection.ReflectionClass::getConstructor)               | Returns the effective constructor descriptor, if present.                 |
| [`getDeclaredMethods()`](thp:std.reflection.ReflectionClass::getDeclaredMethods)       | Returns methods declared by this nominal in deterministic order.          |
| [`getDeclaredMethod()`](thp:std.reflection.ReflectionClass::getDeclaredMethod)         | Returns an exact declared method or null.                                 |
| [`getMethods()`](thp:std.reflection.ReflectionClass::getMethods)                       | Returns effective methods with declared methods first.                    |
| [`getMethod()`](thp:std.reflection.ReflectionClass::getMethod)                         | Returns an exact effective method or throws when absent.                  |
| [`hasMethod()`](thp:std.reflection.ReflectionClass::hasMethod)                         | Reports whether an effective method with the exact name exists.           |
| [`getDeclaredProperties()`](thp:std.reflection.ReflectionClass::getDeclaredProperties) | Returns properties declared by this nominal in deterministic order.       |
| [`getDeclaredProperty()`](thp:std.reflection.ReflectionClass::getDeclaredProperty)     | Returns an exact declared property or null.                               |
| [`getProperties()`](thp:std.reflection.ReflectionClass::getProperties)                 | Returns effective properties with declared properties first.              |
| [`getProperty()`](thp:std.reflection.ReflectionClass::getProperty)                     | Returns an exact effective property or throws when absent.                |
| [`hasProperty()`](thp:std.reflection.ReflectionClass::hasProperty)                     | Reports whether an effective property with the exact name exists.         |
| [`newInstanceArgs()`](thp:std.reflection.ReflectionClass::newInstanceArgs)             | Allocates and constructs an instance with positional or named arguments.  |

## See also

- [Reflection](thp:std.reflection)
