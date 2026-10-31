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
