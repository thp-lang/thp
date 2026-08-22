---
kind: guide
id: guide.languageEnumerations
title: Enumerations
summary: Describes proposed THP enumerations and their engine-provided contracts.
nav:
  section: language
  order: 120
status: experimental
availability: proposed
notice: >-
  Enumeration syntax and runtime contracts are proposed and are not yet backed by an implementation in this checkout.
---

An enumeration defines a closed set of named cases. THP follows PHP's enum
model: a case is a singleton object and cannot carry an associated value.

## Unit cases

```thp
enum Status
{

    case Pending;
    case Complete;
}
```

Use a class when each alternative must contain data. `Option<T>`, for example,
is a standard-library class rather than
an enum because a present option contains a value.

`match` can select behavior by enum case.

## Backed enumerations

A backed enumeration associates each case with a unique integer or string.
The proposed engine contract supplies lookup by backing value.

Every enum is intended to implement
`UnitEnum`. Backed enums also
implement
`BackedEnum`.

The complete syntax for backing types, methods, constants, traits, and
serialization is not yet established.

## See also

- [Control structures](thp:guide.languageControlStructures)
- Predefined interfaces and classes
