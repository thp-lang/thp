---
kind: interface
id: std.spl.SplSubject
title: SplSubject
summary: Manages observers and announces state changes.
name: SplSubject
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

`SplSubject` manages observers and announces state changes.

## Contract

Subjects maintain an observer collection. `notify()` captures the attached observer set when the call begins and invokes every member of that snapshot once. Notification order is implementation-defined; attachments and detachments made by a callback affect only later notifications.

## Example

```thp
$subject->attach($logger);
$subject->notify();
$subject->detach($logger);
```

The observer is notified only while it remains attached.

## See also

- [SPL interfaces](thp:std.iterators)
- [PHP `SplSubject`](https://www.php.net/manual/en/class.splsubject.php)
- [`SplObserver`](thp:std.spl.SplObserver)
