---
kind: interface
id: std.baseTypes.Traversable
title: Traversable
summary: Identifies objects that can be traversed with foreach.
name: Traversable
module: base-types
typeParameters: []
interfaces: []
constants: []
properties: []
status: experimental
availability: proposed
notice: This THP contract is proposed and is not yet implemented in this repository.
version: "0.1"
---

`Traversable` is the marker interface shared by objects that can be consumed by
`foreach`.

## Contract

`Traversable` has no methods. User-defined concrete classes do not implement it
directly; they implement [`Iterator`](thp:std.baseTypes.Iterator) or
[`IteratorAggregate`](thp:std.baseTypes.IteratorAggregate). Code may use
`Traversable` when the key and value types do not need to appear in its
signature.

## Implementations

| Interface                                                  | Notes                                      |
| ---------------------------------------------------------- | ------------------------------------------ |
| [`Iterator`](thp:std.baseTypes.Iterator)                   | The object owns a typed key-value cursor.  |
| [`IteratorAggregate`](thp:std.baseTypes.IteratorAggregate) | The object creates a fresh typed iterator. |

## Example

```thp
function printValues(Traversable $values): void {
    foreach ($values as $value) {
        var_dump($value);
    }
}
```

The parameter accepts either iterator strategy without exposing its concrete
implementation.

## See also

- [`Iterator`](thp:std.baseTypes.Iterator)
- [`IteratorAggregate`](thp:std.baseTypes.IteratorAggregate)
