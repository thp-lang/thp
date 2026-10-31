---
kind: method
id: std.spl.AppendIterator::getIteratorIndex
title: AppendIterator::getIteratorIndex
summary: Returns the active iterator index, or null before traversal or after exhaustion.
name: getIteratorIndex
order: 3
typeParameters: []
parameters: []
returns:
  type: ?int
  description: Returns the active iterator index, or null before traversal or after exhaustion.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: implemented
notice: This method is executable in the reference VM.
version: "0.1"
owner: std.spl.AppendIterator
visibility: public
modifiers: []
---

[`AppendIterator`](thp:std.spl.AppendIterator)`::getIteratorIndex()` returns the active iterator index, or null before traversal.

## Behavior

Returns the active iterator index, or null before traversal.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getIteratorIndex();
```

The call uses the signature and defaults documented above.

## See also

- [`AppendIterator`](thp:std.spl.AppendIterator)
