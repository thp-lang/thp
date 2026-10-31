---
kind: method
id: std.spl.Generator::getReturn
title: Generator::getReturn
summary: Reads the final result after normal generator exhaustion.
name: getReturn
order: 1
typeParameters: []
parameters: []
returns:
  type: mixed
  description: The explicit return value, or null for an implicit end or return without a value.
errors:
  - description: Throws Error before normal exhaustion or after explicit close.
related: []
status: experimental
availability: implemented
notice: This method executes in the standalone VM.
version: "0.7"
owner: std.spl.Generator
visibility: public
modifiers: []
---

`getReturn()` reads the completed generator's final value. It does not move the
cursor. Calling `close()` after normal exhaustion preserves this value.
