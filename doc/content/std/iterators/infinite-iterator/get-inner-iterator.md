---
kind: method
id: std.spl.InfiniteIterator::getInnerIterator
title: InfiniteIterator::getInnerIterator
summary: Returns the iterator used for the current cycle.
name: getInnerIterator
order: 2
typeParameters: []
parameters: []
returns:
  type: Iterator<K, V>
  description: Returns the iterator used for the current cycle.
errors:
  - description: Fails before rewind or after an empty source leaves no current cycle.
related: []
status: experimental
availability: implemented
notice: This method is executable in the reference VM.
version: "0.1"
owner: std.spl.InfiniteIterator
visibility: public
modifiers: []
---

[`InfiniteIterator`](thp:std.spl.InfiniteIterator)`::getInnerIterator()` returns the iterator used for the current cycle.

## Behavior

Returns the iterator used for the current cycle. A cycle must be active.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getInnerIterator();
```

The call uses the signature and defaults documented above.

## See also

- [`InfiniteIterator`](thp:std.spl.InfiniteIterator)
