---
kind: method
id: std.spl.RecursiveFilterIterator::accept
title: RecursiveFilterIterator::accept
summary: Decides whether an entry is yielded.
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
  description: Decides whether an entry is yielded.
errors:
  - description:
      Failures thrown by the callback or comparison operation propagate without
      being wrapped.
related: []
status: experimental
availability: implemented
notice: This experimental recursive filtering member is implemented.
version: "0.1"
owner: std.spl.RecursiveFilterIterator
visibility: public
modifiers:
  - abstract
---

[`RecursiveFilterIterator`](thp:std.spl.RecursiveFilterIterator)`::accept()` decides whether an entry is yielded.

## Behavior

Decides whether an entry is yielded.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->accept($entry);
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveFilterIterator`](thp:std.spl.RecursiveFilterIterator)
