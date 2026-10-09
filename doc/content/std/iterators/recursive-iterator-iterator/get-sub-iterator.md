---
kind: method
id: std.spl.RecursiveIteratorIterator::getSubIterator
title: RecursiveIteratorIterator::getSubIterator
summary: Returns the recursive iterator at a depth.
name: getSubIterator
order: 3
typeParameters: []
parameters:
  - name: level
    type: ?int
    description: Depth to inspect; null selects the current depth.
    default: "null"
returns:
  type: ?RecursiveIterator<K, T>
  description: Returns the recursive iterator at a depth.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: implemented
notice: This experimental traversal member is implemented for recursive cursors.
version: "0.1"
owner: std.spl.RecursiveIteratorIterator
visibility: public
modifiers: []
---

[`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)`::getSubIterator()` returns the recursive iterator at a depth.

## Behavior

Returns the recursive iterator at a depth.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getSubIterator();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)
