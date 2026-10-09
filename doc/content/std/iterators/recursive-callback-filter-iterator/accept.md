---
kind: method
id: std.spl.RecursiveCallbackFilterIterator::accept
title: RecursiveCallbackFilterIterator::accept
summary: Invokes the configured callback.
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
  description: Invokes the configured callback.
errors:
  - description:
      Failures thrown by the callback or comparison operation propagate without
      being wrapped.
related: []
status: experimental
availability: implemented
notice: This experimental recursive filtering member is implemented.
version: "0.1"
owner: std.spl.RecursiveCallbackFilterIterator
visibility: public
modifiers: []
---

[`RecursiveCallbackFilterIterator`](thp:std.spl.RecursiveCallbackFilterIterator)`::accept()` invokes the configured callback.

## Behavior

Invokes the configured callback.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->accept($entry);
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveCallbackFilterIterator`](thp:std.spl.RecursiveCallbackFilterIterator)
