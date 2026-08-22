---
kind: interface
id: std.baseTypes.Stringable
title: Stringable
summary: Defines objects that provide a string representation.
name: Stringable
module: base-types
typeParameters: []
interfaces: []
constants: []
properties: []
status: experimental
availability: proposed
notice: This THP contract is proposed and is not yet implemented in this repository.
version: "0.1"
---

`Stringable` marks an object that can represent itself as a string.

## Contract

`__toString()` returns the object's textual representation. A class that
declares `__toString()` satisfies `Stringable` implicitly, although declaring
the interface explicitly makes the intent clear.

## Example

```thp
class UserId implements Stringable
{

    public function __construct(private int $value)
    {
    }

    public function __toString(): string
    {
        return "user:" . $this->value;
    }
}
```

## See also

- [`Throwable`](thp:std.baseTypes.Throwable)
