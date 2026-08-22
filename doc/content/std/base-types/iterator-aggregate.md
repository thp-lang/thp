---
kind: interface
id: std.baseTypes.IteratorAggregate
title: IteratorAggregate
summary: Defines objects that create a typed keyed iterator for each traversal.
name: IteratorAggregate
module: base-types
typeParameters:
  - name: K
    description: The type of each iterator key.
  - name: V
    description: The type of each iterator value.
interfaces:
  - id: std.baseTypes.Traversable
constants: []
properties: []
status: experimental
availability: proposed
notice: This THP contract is proposed and is not yet implemented in this repository.
version: "0.1"
---

`IteratorAggregate<K, V>` lets an object create a separate typed iterator for
each traversal without storing the cursor on the aggregate itself.

## Contract

`getIterator()` returns a fresh `Iterator<K, V>`. Each `foreach` operation asks
the aggregate for a new iterator and then applies the normal cursor protocol,
including its initial `rewind()`. Separate traversals do not share cursor state.

## Example

```thp
class Users implements IteratorAggregate<int, User>
{
    public function __construct(private map<int, User> $users)
    {
    }

    public function getIterator(): Iterator<int, User>
    {
        return new MapIterator<int, User>($this->users);
    }
}
```

## See also

- [`Traversable`](thp:std.baseTypes.Traversable)
- [`Iterator`](thp:std.baseTypes.Iterator)
