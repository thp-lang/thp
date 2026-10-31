---
kind: method
id: std.spl.Generator::close
title: Generator::close
summary: Unwinds and closes a suspended generator.
name: close
order: 2
typeParameters: []
parameters: []
returns:
  type: void
  description: This callable does not return a value.
errors:
  - description: A throwable from finally or using cleanup propagates.
related: []
status: experimental
availability: implemented
notice: This method executes in the standalone VM.
version: "0.7"
owner: std.spl.Generator
visibility: public
modifiers: []
---

`close()` on a fresh generator skips its body. On a suspended generator it
runs active `using` and `finally` cleanup in nesting order, then discards the
frame. It is idempotent. After normal exhaustion it leaves the return value
available through [`getReturn()`](thp:std.spl.Generator::getReturn).
