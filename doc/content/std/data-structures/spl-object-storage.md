---
kind: class
id: std.spl.SplObjectStorage
title: SplObjectStorage
summary: Associates live object identities with optional information.
name: SplObjectStorage
module: data-structures
typeParameters:
  - name: TInfo
    description: The TInfo type parameter.
interfaces:
  - id: std.baseTypes.IteratorAggregate
    arguments:
      - object
      - ?TInfo
  - id: std.baseTypes.MapAccess
    arguments:
      - object
      - ?TInfo
  - id: std.baseTypes.Countable
constants: []
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`SplObjectStorage` associates live object identities with optional information.

## Construction

| Method                                                       | Description                                  |
| ------------------------------------------------------------ | -------------------------------------------- |
| [`__construct()`](thp:std.spl.SplObjectStorage::__construct) | Creates empty identity-based object storage. |

## Behavior

Keys compare by object identity, not equality of fields. Attaching an existing object replaces its associated information without adding a second entry.

## Errors

Each member page documents the failure conditions relevant to that operation. Concrete THP error classes remain unsettled.

## Example

```thp
$owners = new SplObjectStorage<string>();
$owners->attach($connection, "worker-1");
$name = $owners[$connection];
```

`$name` is associated with that exact `$connection` object.

## See also

- [SPL data structures](thp:std.dataStructures)
- [PHP `SplObjectStorage`](https://www.php.net/manual/en/class.splobjectstorage.php)
