---
kind: class
id: std.spl.ArrayObject
title: ArrayObject
summary: Wraps map-like storage with object methods and configurable iteration.
name: ArrayObject
module: data-structures
typeParameters:
  - name: K
    description: The K type parameter.
  - name: V
    description: The V type parameter.
interfaces:
  - id: std.baseTypes.IteratorAggregate
    arguments:
      - K
      - V
  - id: std.baseTypes.MapAccess
    arguments:
      - K
      - V
  - id: std.baseTypes.Countable
constants:
  - name: STD_PROP_LIST
    type: int
    description: Exposes object properties during property-list operations.
  - name: ARRAY_AS_PROPS
    type: int
    description: Allows stored keys to be accessed as properties.
properties: []
status: experimental
availability: proposed
notice:
  This PHP migration-analysis placeholder is not an accepted THP-native API and
  is not implemented. A typed map wrapper requires a separate THP contract.
version: "0.1"
---

`ArrayObject` records PHP migration behavior for an object around map-like
storage. THP does not have an `array` type, and this name is not part of the
planned native API.

## Construction

| Method                                                  | Description                                                                                              |
| ------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.ArrayObject::__construct) | Wraps the supplied map or public object properties and selects the iterator class used by getIterator(). |

## Behavior

Mutations affect the wrapped storage. Value sorts preserve key associations; key sorts reorder by key. Iterator class names require runtime validation.

## Errors

Each member page documents the failure conditions relevant to that operation. Concrete THP error classes remain unsettled.

## Example

```thp
$settings = new ArrayObject<string, mixed>({"debug" => false});
$settings["debug"] = true;
$copy = $settings->getArrayCopy();
```

`$copy["debug"]` is `true`.

## See also

- [SPL data structures](thp:std.dataStructures)
- [PHP `ArrayObject`](https://www.php.net/manual/en/class.arrayobject.php)
