---
kind: method
id: std.spl.RecursiveRegexIterator::__construct
title: RecursiveRegexIterator::__construct
summary: Wraps a recursive iterator and fixes the transformed value type for the selected mode.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: iterator
    type: RecursiveIterator<K, TIn>
    description: Iterator wrapped or consumed by this operation.
  - name: pattern
    type: string
    description: Pattern used by the operation.
  - name: mode
    type: int
    description: Mode selected from the values documented below.
    default: RecursiveRegexIterator::MATCH
  - name: invert
    type: bool
    description: Value supplied as $invert.
    default: "false"
  - name: preg_flags
    type: int
    description: Flags passed to the regular-expression engine.
    default: "0"
returns:
  type: void
  description: This callable does not return a value.
errors:
  - description:
      Construction fails when an argument violates the documented contract or an
      underlying resource cannot be created. Concrete THP error classes remain
      experimental unless named above.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.spl.RecursiveRegexIterator
visibility: public
modifiers: []
---

[`RecursiveRegexIterator`](thp:std.spl.RecursiveRegexIterator)`::__construct()` wraps a recursive iterator and fixes the transformed value type for the selected mode.

## Behavior

Wraps a recursive iterator and fixes the transformed value type for the selected mode.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new RecursiveRegexIterator($iterator, $pattern);
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveRegexIterator`](thp:std.spl.RecursiveRegexIterator)
