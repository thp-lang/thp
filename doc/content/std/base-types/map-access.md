---
kind: interface
id: std.baseTypes.MapAccess
title: MapAccess
summary: Defines map-style offset operations for objects.
name: MapAccess
module: base-types
typeParameters:
  - name: K
    description: The type accepted as an offset.
  - name: V
    description: The value stored at each offset.
interfaces: []
constants: []
properties: []
status: experimental
availability: proposed
notice: This THP contract is proposed and is not yet implemented in this repository.
version: "0.1"
---

`MapAccess<K, V>` lets an object define how map-style reads, writes, existence
checks, and removals behave.

## Contract

The four methods describe one consistent offset namespace. `offsetExists()`
checks an offset, `offsetGet()` reads it, `offsetSet()` stores a value, and
`offsetUnset()` removes it. Implementations define which offset types and
missing-offset behavior they support.

## Example

```thp
function replace<K, V>(MapAccess<K, V> $values, K $key, V $value): void {
    if ($values->offsetExists($key)) {
        $values->offsetSet($key, $value);
    }
}
```

## See also

- [`Countable`](thp:std.baseTypes.Countable)
