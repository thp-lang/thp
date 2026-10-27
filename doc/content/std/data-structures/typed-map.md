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

`TypedMap<K, V>` is a shared mutable object around an already typed native
`map<K, V>`. Its purpose is shared mutation through object aliases; native maps
already check their key and value types. Prefer a native map for ordinary value
semantics and use this wrapper when aliases must observe updates.

[`__construct()`](thp:std.dataStructures.TypedMap::__construct) starts empty and
takes no initial map. [`set()`](thp:std.dataStructures.TypedMap::set) inserts or
replaces without moving an existing key. [`get()`](thp:std.dataStructures.TypedMap::get)
returns null for absence. If `V` permits null,
[`contains()`](thp:std.dataStructures.TypedMap::contains) distinguishes a stored
null from a missing key. [`remove()`](thp:std.dataStructures.TypedMap::remove)
returns whether an entry was removed;
[`count()`](thp:std.dataStructures.TypedMap::count) counts keys.

[`toMap()`](thp:std.dataStructures.TypedMap::toMap) returns a copy-on-write map
value; later writes to that value and updates to the wrapper do not alter each
other. [`getIterator()`](thp:std.dataStructures.TypedMap::getIterator) returns a
fresh `MapIterator` snapshot preserving key types and insertion order without
coercion. `foreach` uses `IteratorAggregate`. Snapshots share object identities
in keys and values, so object property changes remain visible.

```thp
<?thp
$values = new TypedMap<string, int>();
$alias = $values;
$values->set("first", 1);
$snapshot = $values->toMap();
$alias->set("first", 2);
echo $values->get("first"); // 2: both names refer to one object.
echo $snapshot["first"]; // 1: a native map value keeps its snapshot.
```

Current implementation detail: the wrapper adds an object around native map
storage. Key operations scan entries linearly, and writes and removals copy the
entry sequence. Counts read the stored length; map snapshots share storage until
mutation, and iterator snapshots add a cursor object. These costs are not API
guarantees or evidence of a performance advantage over a native map.
