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
availability: proposed
notice: This THP contract is proposed and is not yet implemented in this repository.
version: "0.1"
---

`Countable` allows an object to provide its element count.

## Contract

`count()` returns the number of elements represented by the object. The value
must be a non-negative integer and should reflect the object's state when the
method is called.

## Example

```thp
function isEmpty(Countable $value): bool {
    return $value->count() === 0;
}
```
