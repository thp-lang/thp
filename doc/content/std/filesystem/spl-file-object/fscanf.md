---
kind: method
id: std.spl.SplFileObject::fscanf
title: SplFileObject::fscanf
summary: Parses the next line into returned values.
name: fscanf
order: 15
typeParameters: []
parameters:
  - name: format
    type: string
    description: Format string used to parse the input.
returns:
  type: vector<mixed>|null
  description: Parses the next line into returned values.
errors:
  - description:
      Underlying I/O failures follow the return sentinel shown in the signature
      or propagate as the experimental THP I/O failure where no sentinel is available.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.spl.SplFileObject
visibility: public
modifiers: []
---

[`SplFileObject`](thp:std.spl.SplFileObject)`::fscanf()` parses the next line into returned values.

## Behavior

Parses the next line into returned values.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->fscanf($format);
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
