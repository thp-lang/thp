---
kind: interface
id: std.spl.RecursiveIterator
title: RecursiveIterator
summary: Extends iteration with access to child sequences.
name: RecursiveIterator
module: iterators
typeParameters:
  - name: K
    description: The type of each current key.
  - name: T
    description: The type of each recursive value.
interfaces:
  - id: std.baseTypes.Iterator
    arguments:
      - K
      - RecursiveEntry<K, T>
constants: []
properties: []
status: experimental
availability: implemented
notice: Implementations are checked through the inherited Iterator cursor methods.
version: "0.1"
---

`RecursiveIterator<K, T>` exposes recursive entries through a keyed cursor.

## Contract

While `valid()` is true, `key()` returns `K` directly and `value()` returns a
`RecursiveEntry<K, T>`. That entry exposes the logical value and its optional
child iterator. Child iterators have independent cursor state and are
unaffected by later advancement of the parent.

## Example

```thp
$iterator->rewind();
if ($iterator->valid()) {
    $entry = $iterator->value();
    echo $entry->value();

    echo $entry->children() !== null;
}
```

The current key comes from the iterator cursor; no key-value wrapper object is
constructed. The child source is obtained from the current recursive entry.

## See also

- [SPL interfaces](thp:std.iterators)
- [`RecursiveEntry`](thp:std.spl.RecursiveEntry)
- [PHP `RecursiveIterator`](https://www.php.net/manual/en/class.recursiveiterator.php)
