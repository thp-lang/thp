---
kind: class
id: std.dataStructures.TypedMap
title: TypedMap
summary: Mutable typed wrapper around an insertion-ordered map.
name: TypedMap
module: data-structures
typeParameters:
  - name: K
    description: Map key type.
  - name: V
    description: Stored value type.
interfaces:
  - id: std.baseTypes.IteratorAggregate
    arguments:
      - K
      - V
  - id: std.baseTypes.Countable
constants: []
properties: []
status: experimental
availability: implemented
notice:
  This experimental native wrapper is implemented. Iterators traverse value
  snapshots; `toMap()` returns a copy-on-write map value.
version: "0.1"
---

`TypedMap<K, V>` starts empty. `set(K $key, V $value): void` inserts or
replaces a value without moving an existing key. `get(K $key): ?V` returns
`null` for a missing key. `contains(K $key): bool`, `remove(K $key): bool`,
and `count(): int` inspect or mutate entries. `toMap(): map<K, V>` returns
the current map value.

`getIterator(): Traversable<K, V>` returns a `MapIterator` snapshot. `foreach`
uses the normal `IteratorAggregate` protocol.

```thp
$values = new TypedMap<string, int>();
$values->set("first", 1);
$values->set("second", 2);
foreach ($values as $key => $value) {
    echo $key;
}
```
