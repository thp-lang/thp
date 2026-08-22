---
kind: method
id: std.spl.RegexIterator::__construct
title: RegexIterator::__construct
summary: Wraps the input iterator and fixes the output type for the selected mode.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: iterator
    type: Iterator<K, TIn>
    description: Iterator wrapped or consumed by this operation.
  - name: pattern
    type: string
    description: Pattern used by the operation.
  - name: mode
    type: int
    description: Mode selected from the values documented below.
    default: RegexIterator::MATCH
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
owner: std.spl.RegexIterator
visibility: public
modifiers: []
---

[`RegexIterator`](thp:std.spl.RegexIterator)`::__construct()` wraps the input iterator and fixes the output type for the selected mode.

## Behavior

Wraps the input iterator and fixes the output type for the selected mode.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new RegexIterator($iterator, $pattern);
```

The call uses the signature and defaults documented above.

## See also

- [`RegexIterator`](thp:std.spl.RegexIterator)
