---
kind: method
id: std.spl.FilterIterator::getInnerIterator
title: FilterIterator::getInnerIterator
summary: Returns the wrapped iterator.
name: getInnerIterator
order: 3
typeParameters: []
parameters: []
returns:
  type: Iterator<K, V>
  description: Returns the wrapped iterator.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: implemented
notice: This experimental accessor executes in the standalone compiler and reference VM.
version: "0.6"
owner: std.spl.FilterIterator
visibility: public
modifiers: []
---

[`FilterIterator`](thp:std.spl.FilterIterator)`::getInnerIterator()` returns the wrapped iterator.

## Behavior

Returns the wrapped iterator.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getInnerIterator();
```

The call uses the signature and defaults documented above.

## See also

- [`FilterIterator`](thp:std.spl.FilterIterator)
