---
kind: method
id: std.spl.FilterIterator::__construct
title: FilterIterator::__construct
summary: Wraps the iterator whose current values are tested by accept().
name: __construct
order: 1
typeParameters: []
parameters:
  - name: iterator
    type: Iterator<K, V>
    description: Iterator wrapped or consumed by this operation.
returns:
  type: void
  description: This callable does not return a value.
errors:
  - description:
      Construction fails when an argument violates the documented contract or an
      underlying resource cannot be created. Concrete THP error classes remain
      experimental unless named above.
related: []
status: experimental
availability: implemented
notice: This constructor executes on concrete subclasses of the abstract FilterIterator.
version: "0.6"
owner: std.spl.FilterIterator
visibility: public
modifiers: []
---

[`FilterIterator`](thp:std.spl.FilterIterator)`::__construct()` wraps the iterator whose current values are tested by accept().

## Behavior

Wraps the iterator whose current values are tested by accept().

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new EvenFilter($iterator);
```

The call uses the signature and defaults documented above.

## See also

- [`FilterIterator`](thp:std.spl.FilterIterator)
