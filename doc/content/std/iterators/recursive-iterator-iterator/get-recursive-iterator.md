---
kind: method
id: std.spl.RecursiveIteratorIterator::getRecursiveIterator
title: RecursiveIteratorIterator::getRecursiveIterator
summary: Returns the wrapped recursive iterator.
name: getRecursiveIterator
order: 4
typeParameters: []
parameters: []
returns:
  type: RecursiveIterator<K, T>
  description: Returns the wrapped recursive iterator.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.spl.RecursiveIteratorIterator
visibility: public
modifiers: []
---

[`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)`::getRecursiveIterator()` returns the wrapped recursive iterator.

## Behavior

Returns the wrapped recursive iterator.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getRecursiveIterator();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)
