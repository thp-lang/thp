---
kind: function
id: std.baseTypes.serialize
title: serialize
summary: Encodes a supported THP value in the version-one binary format.
name: serialize
order: 20
typeParameters: []
parameters:
  - name: value
    type: mixed
    description: Value to encode.
returns:
  type: string
  description: Binary THP serialization document.
errors:
  - description: Throws InvalidArgumentException for unsupported values or format limits.
related: []
status: experimental
availability: implemented
notice: Scalars, vectors, and maps execute; object serialization remains proposed.
version: "0.1"
module: base-types
---

`serialize()` follows the [THP serialization format](thp:guide.serializationFormat).

```thp
$bytes = serialize([1, 2, 3]);
```
