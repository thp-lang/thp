---
kind: class
id: std.spl.SplFixedArray
title: SplFixedArray
summary: Stores values in a contiguous sequence with an explicit size.
name: SplFixedArray
module: data-structures
typeParameters:
  - name: T
    description: The T type parameter.
interfaces:
  - id: std.baseTypes.IteratorAggregate
    arguments:
      - int
      - ?T
  - id: std.baseTypes.MapAccess
    arguments:
      - int
      - ?T
  - id: std.baseTypes.Countable
constants: []
properties: []
status: experimental
availability: proposed
notice:
  This PHP migration-analysis placeholder is not an accepted THP-native API and
  is not implemented. A fixed-size sequence requires a separate THP contract.
version: "0.1"
---

`SplFixedArray` records PHP migration behavior for a fixed-size sequence. THP
does not have an `array` type, and this name is not part of the planned native
API.

## Construction

| Method                                                    | Description                                                                                                                   |
| --------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.SplFixedArray::__construct) | Allocates $size zero-based slots. New slots initially contain null, so the final generic nullability model remains unsettled. |

## Behavior

Indices range from zero through `getSize() - 1`. Growing adds empty slots; shrinking discards values beyond the new size.

`fromVector()` always uses contiguous zero-based positions. Use `fromMap()` when
integer positions from sparse input must be preserved.

## Errors

Each member page documents the failure conditions relevant to that operation. Concrete THP error classes remain unsettled.

## Example

```thp
$values = new SplFixedArray<string>(2);
$values[0] = "left";
$values[1] = "right";
```

Only indices `0` and `1` are available.

## See also

- [SPL data structures](thp:std.dataStructures)
- [PHP `SplFixedArray`](https://www.php.net/manual/en/class.splfixedarray.php)
