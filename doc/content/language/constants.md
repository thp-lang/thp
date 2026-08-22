---
kind: guide
id: guide.languageConstants
title: Constants
summary: Describes THP named values that do not change during execution.
nav:
  section: language
  order: 50
status: experimental
availability: proposed
notice: >-
  Constant declarations and compile-time evaluation rules remain proposed and are not implemented in this repository.
---

Constants associate a name with a value that cannot be reassigned. THP reserves
PHP-shaped forms for global constants, class constants, and enumeration cases,
but their complete validation and evaluation rules remain proposed.

## Declaration scopes

- A global constant belongs to its namespace.
- A class constant belongs to a class or interface.
- An enumeration case belongs to its enumeration.

Class constants and enumeration cases use `TypeName::MEMBER` access syntax.

```thp
class Limits
{

    public const MAX_RETRIES = 3;
}

$limit = Limits::MAX_RETRIES;
```

Until constant expressions are specified, do not assume that every expression
accepted by PHP is valid in a THP constant initializer.

## See also

- [Namespaces](thp:guide.languageNamespaces)
- [Classes and objects](thp:guide.languageClassesAndObjects)
- [Enumerations](thp:guide.languageEnumerations)
