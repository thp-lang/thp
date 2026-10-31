---
kind: class
id: std.extensions.RegexCapture
title: RegexCapture
summary: A captured byte string and its subject byte offset.
name: RegexCapture
module: extensions
typeParameters: []
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
notice: Created by PCRE2 match and split operations when offset capture is selected; direct construction is unavailable.
version: "0.11"
---

`RegexCapture` carries nullable text and a byte offset. An unmatched group has
null text and offset `-1`.

| Method                                                | Result  |
| ----------------------------------------------------- | ------- |
| [`text()`](thp:std.extensions.RegexCapture::text)     | `string | null` |
| [`offset()`](thp:std.extensions.RegexCapture::offset) | `int`   |
