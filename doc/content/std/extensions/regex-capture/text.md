---
kind: method
id: std.extensions.RegexCapture::text
title: RegexCapture::text
summary: Returns captured bytes or null for an unmatched group.
name: text
order: 1
typeParameters: []
parameters: []
returns:
  type: "string|null"
  description: Captured bytes or null.
errors: []
related: []
status: experimental
availability: implemented
notice: This method is executable in the reference VM.
version: "0.11"
owner: std.extensions.RegexCapture
visibility: public
modifiers: []
---

`text()` returns the original bytes of a participating capture.
