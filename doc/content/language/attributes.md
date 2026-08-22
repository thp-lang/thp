---
kind: guide
id: guide.languageAttributes
title: Attributes
summary: Describes proposed THP declaration metadata syntax.
nav:
  section: language
  order: 160
status: experimental
availability: proposed
notice: >-
  Attribute syntax and reflection behavior are proposed and are not yet backed by an implementation in this checkout.
---

Attributes attach structured metadata to declarations with `#[...]` syntax.
An attribute name refers to a class marked with the
`Attribute` meta-attribute.

```thp
#[Deprecated("Use renderDocument() instead")]
function renderLegacy(Document $document): string {
    return renderDocument($document);
}
```

## Proposed model

Attribute arguments are intended to be evaluated as constant expressions.
Targets and repeatability are configured by the attribute class. Consumers
would inspect metadata through a future reflection API or through compiler
behavior defined for a predefined attribute.

Exact validation timing, reflection objects, inheritance, and repeated
attributes are not yet established.

## See also

- Predefined attributes
- [Classes and objects](thp:guide.languageClassesAndObjects)
- [Constants](thp:guide.languageConstants)
