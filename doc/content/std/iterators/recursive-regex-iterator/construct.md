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
    description: Constant match or split flags for the selected mode.
    default: "0"
  - name: replacement
    type: string
    description: Replacement template for REPLACE mode.
    default: '""'
returns:
  type: void
  description: This callable does not return a value.
errors:
  - description:
      Invalid PCRE2 patterns fail at construction. Nonconstant modes or flags,
      unsupported flag combinations, and mismatched generic output types are
      compile errors.
related: []
status: experimental
availability: implemented
notice: Fixed mode and flag contracts execute in the reference VM.
version: "0.11"
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
