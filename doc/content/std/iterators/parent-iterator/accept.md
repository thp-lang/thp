---
kind: method
id: std.spl.ParentIterator::accept
title: ParentIterator::accept
summary: Returns whether the entry has children.
name: accept
order: 2
typeParameters: []
parameters:
  - name: value
    type: RecursiveEntry<K, T>
    description: Complete recursive entry.
  - name: key
    type: K
    description: Key of the entry.
returns:
  type: bool
  description: Returns whether the entry has children.
errors:
  - description:
      Failures thrown by the callback or comparison operation propagate without
      being wrapped.
related: []
status: experimental
availability: implemented
notice: This experimental recursive filtering member is implemented.
version: "0.1"
owner: std.spl.ParentIterator
visibility: public
modifiers: []
---

[`ParentIterator`](thp:std.spl.ParentIterator)`::accept()` returns whether the entry has children.

## Behavior

Returns whether the entry has children.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->accept($entry);
```

The call uses the signature and defaults documented above.

## See also

- [`ParentIterator`](thp:std.spl.ParentIterator)
