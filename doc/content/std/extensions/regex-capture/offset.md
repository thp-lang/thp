---
kind: method
id: std.extensions.RegexCapture::offset
title: RegexCapture::offset
summary: Returns the capture's byte offset or -1 when unmatched.
name: offset
order: 2
typeParameters: []
parameters: []
returns:
  type: int
  description: Zero-based byte offset, or -1.
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

`offset()` counts bytes from the beginning of the subject.
