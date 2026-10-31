---
kind: method
id: std.spl.CallbackFilterIterator::accept
title: CallbackFilterIterator::accept
summary: Invokes the callback for the current value.
name: accept
order: 2
typeParameters: []
parameters:
  - name: value
    type: V
    description: Current inner value.
  - name: key
    type: K
    description: Current inner key.
returns:
  type: bool
  description: Invokes the callback for the current value.
errors:
  - description:
      Failures thrown by the callback or comparison operation propagate without
      being wrapped.
related: []
status: experimental
availability: implemented
notice: This experimental callback dispatch executes in the standalone compiler and reference VM.
version: "0.6"
owner: std.spl.CallbackFilterIterator
visibility: public
modifiers: []
---

[`CallbackFilterIterator`](thp:std.spl.CallbackFilterIterator)`::accept()` invokes the callback for the current value.

## Behavior

Invokes the callback for the current value.

The stored callback receives `V` then `K` and returns `bool`. A direct call
does not advance the cursor. A callback exception or allocation failure
propagates unchanged.

## Example

```thp
$result = $instance->accept($value, $key);
```

The call uses the signature and defaults documented above.

## See also

- [`CallbackFilterIterator`](thp:std.spl.CallbackFilterIterator)
