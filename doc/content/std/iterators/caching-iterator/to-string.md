---
kind: method
id: std.spl.CachingIterator::__toString
title: CachingIterator::__toString
summary: Formats the current entry according to flags.
name: __toString
order: 4
typeParameters: []
parameters: []
returns:
  type: string
  description: Formats the current entry according to flags.
errors:
  - description:
      Fails on an exhausted cursor or a selected value that is neither a
      scalar nor Stringable.
related: []
status: experimental
availability: implemented
notice: This method is executable in the reference VM.
version: "0.1"
owner: std.spl.CachingIterator
visibility: public
modifiers: []
---

[`CachingIterator`](thp:std.spl.CachingIterator)`::__toString()` formats the current entry according to flags.

## Behavior

Formats the current entry according to flags.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->__toString();
```

The call uses the signature and defaults documented above.

## See also

- [`CachingIterator`](thp:std.spl.CachingIterator)
