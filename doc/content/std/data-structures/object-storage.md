---
kind: class
id: std.dataStructures.ObjectStorage
title: ObjectStorage
summary: Associates data with object identities.
name: ObjectStorage
module: data-structures
typeParameters:
  - name: T
    description: Attached data type.
interfaces:
  - id: std.baseTypes.IteratorAggregate
    arguments:
      - int
      - mixed
  - id: std.baseTypes.Countable
constants: []
properties: []
status: experimental
availability: implemented
notice: This experimental native object storage is implemented. Object keys are
  validated at runtime because THP has no top-level object type constraint.
version: "0.1"
---

`ObjectStorage<T>` associates one `T` value with each object identity.
`attach(mixed $object, T $data): void` replaces data for an existing object
without moving it. `get(mixed $object): ?T`, `contains(mixed $object): bool`,
and `detach(mixed $object): bool` use identity rather than object properties.
Non-object keys throw `TypeError`. `count(): int` returns the number of objects.

`getIterator(): Traversable<int, mixed>` returns an insertion-order snapshot of
attached objects. `foreach` uses the normal `IteratorAggregate` protocol.

```thp
class Box {}
$box = new Box();
$storage = new ObjectStorage<string>();
$storage->attach($box, "ready");
echo $storage->contains($box);
```
