---
kind: function
id: std.baseTypes.unserialize
title: unserialize
summary: Decodes one version-one THP binary serialization document.
name: unserialize
order: 21
typeParameters: []
parameters:
  - name: value
    type: string
    description: Binary THP serialization document.
returns:
  type: mixed
  description: Decoded request-owned value.
errors:
  - description: Throws UnexpectedValueException for malformed or oversized input.
related: []
status: experimental
availability: implemented
notice: Scalars, vectors, and maps execute; object serialization remains proposed.
version: "0.1"
module: base-types
---

`unserialize()` follows the [THP serialization format](thp:guide.serializationFormat).
It accepts one value and rejects trailing bytes.

```thp
$value = unserialize(serialize(42));
```
