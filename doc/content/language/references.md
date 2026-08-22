---
kind: guide
id: guide.languageReferences
title: References
summary: Describes the status of PHP-shaped reference aliasing in THP.
nav:
  section: language
  order: 170
status: experimental
availability: proposed
notice: >-
  Reference assignment, lifetime, and execution semantics are not part of the stable THP contract.
---

A reference would make multiple variable names alias the same storage rather
than copy or independently bind a value. The PHP-shaped `&` syntax below is
reserved for design discussion and is not a supported runtime contract.

```thp
$alias =& $value;
```

THP code should use ordinary values and explicit mutable objects or collections
where shared mutation is required.

## See also

- [Variables](thp:guide.languageVariables)
- [Functions](thp:guide.languageFunctions)
- [Classes and objects](thp:guide.languageClassesAndObjects)
