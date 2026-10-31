---
kind: class
id: std.spl.MapIterator
title: MapIterator
summary: Rewindable cursor over an insertion-ordered map snapshot.
name: MapIterator
module: iterators
typeParameters:
  - name: K
    description: The map key type.
  - name: V
    description: The map value type.
interfaces:
  - id: std.baseTypes.Iterator
    arguments:
      - K
      - V
constants: []
properties: []
status: experimental
availability: implemented
notice: This experimental native iterator is implemented in the standalone compiler and VM.
version: "0.5"
---

`MapIterator<K, V>` captures a copy-on-write snapshot of a `map<K, V>` in
insertion order. Mutating the original map after construction does not change
its keys or values.

[`__construct()`](thp:std.spl.MapIterator::__construct) accepts the map.
The cursor starts at the first entry. `rewind()` returns there, `advance()`
moves toward exhaustion, and `key()` and `value()` fail after exhaustion.

```thp
$source: map<string, int> = {"a" => 1, "b" => 2};
$cursor = new MapIterator($source);
$source["a"] = 9;
echo $cursor->value(); // 1
```

A map converts to a fresh `MapIterator<K, V>` at a compatible typed iterator
boundary. Direct native `foreach` retains insertion order through its
collection traversal path.
