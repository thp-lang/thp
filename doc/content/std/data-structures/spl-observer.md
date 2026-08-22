---
kind: interface
id: std.spl.SplObserver
title: SplObserver
summary: Receives state-change notifications from an SPL subject.
name: SplObserver
module: data-structures
typeParameters: []
interfaces: []
constants: []
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired interface contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`SplObserver` receives state-change notifications from an SPL subject.

## Contract

The subject calls `update()` for each attached observer during notification. The observer reads any event state from the subject; this minimal PHP-shaped contract does not carry a separate event value.

## Example

```thp
class Logger implements SplObserver
{

    public function update(SplSubject $subject): void
    {
        print("subject changed");
    }
}
```

The observer receives the subject that initiated notification.

## See also

- [SPL interfaces](thp:std.iterators)
- [PHP `SplObserver`](https://www.php.net/manual/en/class.splobserver.php)
- [`SplSubject`](thp:std.spl.SplSubject)
