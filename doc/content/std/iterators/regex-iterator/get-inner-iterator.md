---
kind: method
id: std.spl.RegexIterator::getInnerIterator
title: RegexIterator::getInnerIterator
summary: Returns the wrapped input iterator.
name: getInnerIterator
order: 5
typeParameters: []
parameters: []
returns:
  type: Iterator<K, TIn>
  description: The original input cursor.
errors: []
related: []
status: experimental
availability: implemented
notice: This method executes in the reference VM.
version: "0.11"
owner: std.spl.RegexIterator
visibility: public
modifiers: []
---

`getInnerIterator()` returns the wrapped cursor without rewinding it.
