---
kind: guide
id: guide.languageReflection
title: Reflection
summary: Inspects retained program types and declarations through immutable VM descriptors.
nav:
  section: language
  order: 115
status: experimental
availability: implemented
notice: >-
  Nominal, callable, property, parameter, and derived type descriptors execute
  in the reference VM. Attributes, source locations, constants, closures,
  modifier bitmasks, dynamic loading, and constructor-less allocation remain
  proposed.
---

THP reflection uses PHP-shaped constructors and descriptor names, but operates
only on metadata retained in the verified linked program. Descriptor state is
initialized once by the VM and cannot be mutated afterward.

## Construction and discovery

Five concrete descriptors can be constructed directly:

```thp
$class = new ReflectionClass("App\\Model\\User");
$function = new ReflectionFunction("App\\format");
$method = new ReflectionMethod($user, "displayName");
$property = new ReflectionProperty($user, "name");
$parameter = new ReflectionParameter([$user, "rename"], "name");
```

Their signatures are:

- `ReflectionClass::__construct(object|string $objectOrClass)`
- `ReflectionFunction::__construct(string $function)`
- `ReflectionMethod::__construct(object|string $objectOrClass, string $method)`
- `ReflectionProperty::__construct(object|string $objectOrClass, string $property)`
- `ReflectionParameter::__construct(string|vector<mixed> $function, int|string $parameter)`

The parameter callable vector must be exactly `[object|string, string]`.
Supplying an object preserves its concrete generic arguments. String class
lookup produces an erased descriptor, so `getType()` returns `null` for a
generic class found by name. Names are canonical and case-sensitive and may
begin with one optional `\\`.

Discovery performs no source-provider, filesystem, module-execution, include,
or autoload work. This also applies when a frozen program runs without source
files. A missing class, function, method, property, or parameter throws
`ReflectionException`; an invalid dynamic argument shape throws `TypeError`.

Type descriptors are obtained from reflected properties, parameters, return
types, and class types. `ReflectionType` and `ReflectionFunctionAbstract` are
abstract, while `ReflectionNamedType` and `ReflectionUnionType` are produced by
the VM. Nullable forms are normalized, so `?string` and `string|null` compare
equal.

## Members and ordering

Declared collections contain members whose declaring nominal is exactly the
reflected nominal, including surviving trait-composed members. Class-body
members retain source order. Trait contributions and aliases retain their
composition order.

Effective collections begin with declared members, then visit ancestors
nearest-first and add visible names not already selected. Inherited private
members are excluded. A parent-private member remains discoverable from the
parent descriptor, even when a child privately redeclares the same name.
Repeated enumeration is deterministic and independent of map iteration.

Reflected members are privileged. Visibility predicates describe the retained
declaration, but visibility does not prevent reflective access. Ordinary source
access remains subject to lexical visibility checks.

## Checked access and calls

Property descriptors use the exact retained property slot. They validate the
receiver class, reified generic arguments, and assigned value type without a
second name lookup.

`invokeArgs()` accepts either a positional `vector<mixed>` or a case-sensitive
named `map<string, mixed>`. Positional values bind in declaration order. Named
values may reorder parameters, apply retained defaults, reject unknown names,
and cannot target a variadic parameter. Variadics pack remaining positional
values.

A method descriptor invokes its exact retained callee rather than performing a
new name-based virtual lookup. Instance methods require a compatible object;
static methods ignore the supplied receiver. Constructor descriptors may be
invoked on compatible existing objects. Exceptions thrown by invoked code
propagate unchanged.

## Construction

`newInstanceArgs()` allocates through the ordinary object allocator, records
concrete generic arguments, initializes flattened defaults, binds constructor
arguments, and invokes the effective constructor. Interfaces, traits, abstract
classes, incomplete generic descriptors, and runtime-owned native classes throw
`Error`. A missing or non-public constructor throws `ReflectionException`.
Wrong dynamic argument types throw `TypeError`, wrong argument counts throw
`ArgumentCountError`, and unknown named arguments throw `Error`. Reflection
does not expose constructor-less allocation.

## Intentional differences from PHP

- Lookups are case-sensitive and never trigger runtime autoloading.
- Nullable results use `null`, not PHP's `false`.
- Positional and named argument collections use vectors and maps, not PHP arrays.
- Declared and effective member APIs are explicit and deterministic.
- Types compare semantically and expose generic arguments and type parameters.
- Nominals report module ownership; composed members retain trait origins.
- Reflection never triggers runtime loading and has no `setAccessible()`.
- Closures, the deprecated one-argument `ReflectionMethod` form, modifier
  filters, and unlisted PHP Reflection APIs are not implemented.

## See also

- [Reflection API](thp:std.reflection)
- [Classes and objects](thp:guide.languageClassesAndObjects)
- [Types](thp:guide.languageTypes)
