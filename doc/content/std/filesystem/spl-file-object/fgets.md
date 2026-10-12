---
kind: method
id: std.spl.SplFileObject::fgets
title: SplFileObject::fgets
summary: Reads the next line.
name: fgets
order: 3
typeParameters: []
parameters: []
returns:
  type: string
  description: Reads the next line.
errors:
  - description:
      Underlying I/O failures follow the return sentinel shown in the signature
      or propagate as the experimental THP I/O failure where no sentinel is available.
related: []
status: experimental
availability: implemented
notice: Reads the next line from the owned read-only file.
version: "0.1"
owner: std.spl.SplFileObject
visibility: public
modifiers: []
---

[`SplFileObject`](thp:std.spl.SplFileObject)`::fgets()` reads the next line.

## Behavior

Reads the next line.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->fgets();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
