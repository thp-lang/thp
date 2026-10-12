---
kind: method
id: std.spl.RecursiveRegexIterator::getInnerIterator
title: RecursiveRegexIterator::getInnerIterator
summary: Returns the wrapped recursive input iterator.
name: getInnerIterator
order: 2
typeParameters: []
parameters: []
returns:
  type: RecursiveIterator<K, TIn>
  description: The original recursive cursor.
errors: []
related: []
status: experimental
availability: implemented
notice: This method executes in the reference VM.
version: "0.11"
owner: std.spl.RecursiveRegexIterator
visibility: public
modifiers: []
---

`getInnerIterator()` returns the wrapped cursor without rewinding it.
