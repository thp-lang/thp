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
    arguments:
      - K
      - V
constants: []
properties: []
status: experimental
availability: proposed
notice: This THP contract is proposed and is not yet implemented in this repository.
version: "0.1"
---

`IteratorAggregate<K, V>` invariantly extends `Traversable<K, V>` and lets an
object create a separate typed traversal source without storing the cursor on
the aggregate itself. `K` has no additional constraint.

## Contract

`getIterator()` returns `Traversable<K, V>`. Each `foreach` operation calls it
once for that aggregate layer. If it returns another aggregate, dispatch
continues one layer at a time; when it reaches a direct `Iterator<K, V>`,
`foreach` calls `rewind()` before `valid()`. Separate traversals normally return
independent sources, but freshness is not part of the return type.

## Example

```thp
class Users implements IteratorAggregate<int, User>
{
    public function __construct(private map<int, User> $users)
    {
    }

    public function getIterator(): Traversable<int, User>
    {
        return new MapIterator<int, User>($this->users);
    }
}
```

## See also

- [`Traversable`](thp:std.baseTypes.Traversable)
- [`Iterator`](thp:std.baseTypes.Iterator)
