---
kind: class
id: std.spl.IteratorIterator
title: IteratorIterator
summary: Adapts a cursor iterator to the outer-iterator contract.
name: IteratorIterator
module: iterators
typeParameters:
  - name: K
    description: The key type preserved from the wrapped iterator.
  - name: V
    description: The value type preserved from the wrapped iterator.
interfaces:
  - id: std.spl.OuterIterator
    arguments:
      - K
      - V
constants: []
properties: []
status: experimental
availability: implemented
notice: This experimental contract is implemented in the standalone compiler and VM.
version: "0.5"
---

`IteratorIterator` adapts a cursor iterator to the outer-iterator contract.

## Construction

| Method                                                                 | Description                                                                                 |
| ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.IteratorIterator::__construct)           | Wraps the supplied cursor iterator without changing its key, value, or exhaustion behavior. |
| [`getInnerIterator()`](thp:std.spl.IteratorIterator::getInnerIterator) | Returns the same wrapped iterator object.                                                   |

## Behavior

The adapter forwards values from the wrapped iterator without changing their
order or exhaustion behavior. Call `getIterator()` on an aggregate before
constructing this adapter.

## Errors

Construction validates the parameters shown above. Cursor operations call the same method on the wrapped iterator and propagate its failures unchanged.

## Example

```thp
$adapter = new IteratorIterator(new VectorIterator([1, 2]));
$inner = $adapter->getInnerIterator();
```

The wrapped iterator remains inspectable.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `IteratorIterator`](https://www.php.net/manual/en/class.iteratoriterator.php)
