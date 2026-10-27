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

`ObjectStorage<T>` is an optional object-only identity association whose
iteration yields attached objects. Native object-keyed maps already associate
values by object identity: prefer `map<Box, T>` when typed keys and native map
value semantics fit. Choose this wrapper when shared object mutation, runtime
rejection of every non-object key, and iteration over objects are useful.
THP currently has no top-level object type constraint, so key parameters are
`mixed` and validated at runtime.

[`__construct()`](thp:std.dataStructures.ObjectStorage::__construct) starts empty.
[`attach()`](thp:std.dataStructures.ObjectStorage::attach) requires both an object
and its data; reattaching replaces data without moving the object.
[`get()`](thp:std.dataStructures.ObjectStorage::get) returns null for absence;
[`contains()`](thp:std.dataStructures.ObjectStorage::contains) distinguishes an
absent object from one with null data. [`detach()`](thp:std.dataStructures.ObjectStorage::detach)
returns whether an object was removed. All four key operations reject non-objects
with `TypeError`, even for lookups or removals. Matching uses identity, never
object properties. [`count()`](thp:std.dataStructures.ObjectStorage::count) counts
attached identities.

The storage strongly retains both attached objects and data. Detaching releases
its references; other references can keep either alive. This is not weak-key
storage. [`getIterator()`](thp:std.dataStructures.ObjectStorage::getIterator)
returns a fresh insertion-order `VectorIterator` snapshot of the objects, with
dense integer keys. It does not yield attached data. Snapshots retain their
objects after detachment and share their identity and property mutations.
`foreach` uses `IteratorAggregate`.

```thp
<?thp
class Box {}
$box = new Box();
$alias = $box;
$other = new Box();
$storage = new ObjectStorage<string>();
$storage->attach($box, "ready");
echo $storage->contains($alias); // true: same identity.
echo $storage->contains($other); // false: distinct identity.
```

Current implementation detail: an object wrapper owns a vector of object/data
records. Key operations scan identities linearly, and attachments and detachments
copy the record sequence. Counting reads the stored length; capturing iteration
allocates a vector of object references and a cursor. These costs are not API
guarantees or measured advantages over object-keyed maps.
