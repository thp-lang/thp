---
kind: interface
id: std.baseTypes.Traversable
title: Traversable
summary: Identifies objects that can be traversed with foreach.
name: Traversable
module: base-types
typeParameters:
  - name: K
    description: The invariant type of each traversal key; no additional key constraint applies.
  - name: V
    description: The invariant type of each traversal value.
interfaces: []
constants: []
properties: []
status: experimental
availability: proposed
notice: This THP contract is proposed and is not yet implemented in this repository.
version: "0.1"
---

`Traversable<K, V>` is the invariant marker interface shared by objects that
can be consumed by `foreach`. `K` may be any THP type; unlike PHP array keys,
iterator keys are not coerced to `int` or `string`.

## Contract

`Traversable<K, V>` has no methods. A concrete class must choose exactly one
strategy: implement [`Iterator<K, V>`](thp:std.baseTypes.Iterator) directly or
implement
[`IteratorAggregate<K, V>`](thp:std.baseTypes.IteratorAggregate). A concrete
class that declares `Traversable<K, V>` directly, or implements both
strategies, is a compile error. An abstract class may implement `Traversable`
directly and defer that choice to each concrete subclass.

The type parameters are invariant. For example, a
`Traversable<int, Dog>` is not substitutable for a
`Traversable<int, Animal>` even when `Dog` is a subtype of `Animal`.

## Implementations

| Interface                                                  | Notes                                                |
| ---------------------------------------------------------- | ---------------------------------------------------- |
| [`Iterator`](thp:std.baseTypes.Iterator)                   | The object owns a typed key-value cursor.            |
| [`IteratorAggregate`](thp:std.baseTypes.IteratorAggregate) | The object returns the next typed traversable layer. |

## Example

```thp
function printValues<K, V>(Traversable<K, V> $values): void {
    foreach ($values as $value) {
        var_dump($value);
    }
}
```

The proposed parameter accepts either iterator strategy without exposing its
concrete implementation. Iterator-object `foreach` is not executable yet.

These concrete declarations are invalid:

```thp
class MissingStrategy<K, V> implements Traversable<K, V> {}

class Ambiguous<K, V>
    implements Iterator<K, V>, IteratorAggregate<K, V>
{
    // Invalid even if every required method is present.
}
```

## See also

- [`Iterator`](thp:std.baseTypes.Iterator)
- [`IteratorAggregate`](thp:std.baseTypes.IteratorAggregate)
