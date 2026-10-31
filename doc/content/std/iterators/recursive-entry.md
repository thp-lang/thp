---
kind: class
id: std.spl.RecursiveEntry
title: RecursiveEntry
summary: Carries a value and the optional iterator for its children.
name: RecursiveEntry
module: iterators
typeParameters:
  - name: K
    description: The key type shared by parent and child cursors.
  - name: T
    description: The type of each recursive value.
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
notice: This immutable entry executes in the reference VM.
version: "0.1"
---

This is a final class.

`RecursiveEntry<K, T>` keeps a recursive value and its children in one immutable
pull result.

Construct it with a value and optional `RecursiveIterator<K, T>` child
cursor. The child cursor is retained independently of the parent cursor.

## Behavior

The value and child source remain paired after the parent iterator advances.
Consumers never query child state through an implicit current cursor.

## Example

```thp
$leaf = new RecursiveEntry<int, int>(4);
echo $leaf->children() === null;
```

## See also

- [`RecursiveIterator`](thp:std.spl.RecursiveIterator)
- [THP `Iterator`](thp:std.baseTypes.Iterator)
