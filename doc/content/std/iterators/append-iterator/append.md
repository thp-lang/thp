---
kind: method
id: std.spl.AppendIterator::append
title: AppendIterator::append
summary: Appends an iterator to the sequence.
name: append
order: 2
typeParameters: []
parameters:
  - name: iterator
    type: Iterator<K, V>
    description: Iterator wrapped or consumed by this operation.
returns:
  type: void
  description: This method does not return a value.
errors:
  - description:
      Throws LogicException if traversal has started. Allocation failure while
      growing the iterator list also propagates.
related: []
status: experimental
availability: implemented
notice: This method is executable in the reference VM.
version: "0.1"
owner: std.spl.AppendIterator
visibility: public
modifiers: []
---

[`AppendIterator`](thp:std.spl.AppendIterator)`::append()` appends an iterator to the sequence.

## Behavior

Appends an iterator to the sequence.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->append($iterator);
```

The call uses the signature and defaults documented above.

## See also

- [`AppendIterator`](thp:std.spl.AppendIterator)
