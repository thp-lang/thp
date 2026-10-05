---
kind: interface
id: std.baseTypes.Countable
title: Countable
summary: Defines objects whose elements can be counted.
name: Countable
module: base-types
typeParameters: []
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
notice: The compiler and reference VM implement this experimental contract.
version: "0.1"
---

`Countable` allows an object to provide its element count.

## Contract

`count()` returns the number of elements represented by the object. The value
must be a non-negative integer and should reflect the object's state when the
method is called.
The VM rejects a negative result with `UnexpectedValueException`, including
when `count($object)` calls the method. `count()` also accepts strings, vectors,
and maps through their native length operation.

## Example

```thp
function isEmpty(Countable $value): bool {
    return $value->count() === 0;
}
```
