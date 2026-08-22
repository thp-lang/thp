---
kind: interface
id: std.baseTypes.Serializable
title: Serializable
summary: Defines the legacy custom object serialization contract.
name: Serializable
module: base-types
typeParameters: []
interfaces: []
constants: []
properties: []
status: experimental
availability: proposed
notice: This legacy THP contract is proposed and is not yet implemented in this repository.
version: "0.1"
---

`Serializable` lets an object control the payload used by legacy serialization.

## Contract

`serialize()` produces the object's serialized payload. During restoration,
`unserialize()` receives that payload and reconstructs the object's state.
Restoration invokes `unserialize()` instead of the normal constructor.

New THP APIs should prefer `__serialize()` and `__unserialize()`. This interface
exists for compatibility with the legacy serialization mechanism.

## Example

```thp
class Identifier implements Serializable
{

    public function serialize(): ?string
    {
        return $this->value;
    }

    public function unserialize(string $data): void
    {
        $this->value = $data;
    }
}
```
