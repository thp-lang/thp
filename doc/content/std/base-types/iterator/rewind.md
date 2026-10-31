---
kind: method
id: std.baseTypes.Iterator::rewind
title: Iterator::rewind
summary: Starts traversal according to the iterator’s restart policy.
name: rewind
order: 1
typeParameters: []
parameters: []
returns:
  type: void
  description: This method does not return a value.
errors:
  - description: Failures encountered while restarting the underlying source propagate.
related: []
status: experimental
availability: implemented
notice: This method signature participates in executable interface checks and virtual dispatch.
version: "0.1"
owner: std.baseTypes.Iterator
visibility: public
modifiers: []
---

[`Iterator`](thp:std.baseTypes.Iterator)`::rewind()` starts traversal according to
the implementation’s documented restart policy.

## Behavior

After a successful rewind of an ordinary restartable iterator, `valid()` returns
`true` when an element exists and `false` when the source is empty. Restartable
iterators return to their first element after partial or complete traversal. A one-shot implementation
may fail when called after it has advanced, but it must accept the initial
`rewind()` performed on a fresh iterator.
An explicitly documented no-rewind adapter such as
[`NoRewindIterator`](thp:std.spl.NoRewindIterator) may instead leave its shared
cursor unchanged, including when partially consumed or exhausted.
Direct iterator `foreach` calls this exactly once before its first `valid()`
check. Aggregate layers are resolved first, with one `getIterator()` call per
layer.

## Example

```thp
$iterator->rewind();

if ($iterator->valid()) {
    echo $iterator->value();
}
```

## See also

- [`Iterator`](thp:std.baseTypes.Iterator)
- [`valid()`](thp:std.baseTypes.Iterator::valid)
