---
kind: method
id: std.spl.OuterIterator::getInnerIterator
title: OuterIterator::getInnerIterator
summary: Returns the wrapped iterator.
name: getInnerIterator
order: 1
typeParameters: []
parameters: []
returns:
  type: Iterator<K, V>
  description: Returns the wrapped iterator.
errors:
  - description:
      Fails if the adapter has no active inner iterator; delegated failures
      also propagate.
related: []
status: experimental
availability: implemented
notice: This experimental contract is implemented in the standalone compiler and VM.
version: "0.5"
owner: std.spl.OuterIterator
visibility: public
modifiers: []
---

[`OuterIterator`](thp:std.spl.OuterIterator)`::getInnerIterator()` returns the wrapped iterator.

## Behavior

Returns the wrapped iterator.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getInnerIterator();
```

The call uses the signature and defaults documented above.

## See also

- [`OuterIterator`](thp:std.spl.OuterIterator)
