---
kind: function
id: std.spl.iterator_apply
title: iterator_apply
summary: Calls a callback for each remaining iterator value.
name: iterator_apply
order: 5
typeParameters:
  - name: K
    description: The iterator key type.
  - name: T
    description: The iterator value type.
parameters:
  - name: iterator
    type: Iterator<K, T>
    description: Cursor iterator to advance through its remaining values.
  - name: callback
    type: callable<T, K, bool>
    description: Receives the current value then key and decides whether to continue.
returns:
  type: int
  description: The number of callback invocations.
errors:
  - description: Cursor, callback, and allocation failures propagate and stop traversal.
related: []
status: experimental
availability: implemented
notice: This experimental contract executes in the standalone compiler and reference VM.
version: "0.6"
module: iterators
---

`iterator_apply()` calls a callback for each remaining iterator value.

## Behavior

The callback receives the current `T` value followed by its `K` key. The
function counts the call, advances the cursor, then stops if the callback
returned `false`; otherwise it continues while `valid()` is true. It never
rewinds. An empty iterator returns zero without calling the callback. The
iterator's declared `K` and `T` determine callback types; an untyped empty
collection literal cannot establish an iterator shape without an expected type.
No result keys are produced, and the underlying cursor keeps its own keys.

A callback exception or cursor failure propagates without another advance of
the failing entry. Allocation failure in the callback or cursor also
propagates. The count is not returned after a failure.

## Example

```thp
$calls = iterator_apply($iterator, function (string $value, int $key): bool {
    echo $value;
    return true;
});
```

## See also

- [SPL functions](thp:std.dataStructures)
- [PHP `iterator_apply()`](https://www.php.net/manual/en/function.iterator-apply.php)
