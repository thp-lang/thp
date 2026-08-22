---
kind: method
id: std.spl.RecursiveCallbackFilterIterator::accept
title: RecursiveCallbackFilterIterator::accept
summary: Invokes the configured callback.
name: accept
order: 2
typeParameters: []
parameters:
  - name: entry
    type: RecursiveEntry<T>
    description: Value supplied as $entry.
returns:
  type: bool
  description: Invokes the configured callback.
errors:
  - description:
      Failures thrown by the callback or comparison operation propagate without
      being wrapped.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
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
