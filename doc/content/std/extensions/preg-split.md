---
kind: function
id: std.extensions.preg_split
title: preg_split
summary: Splits a binary string on PCRE2 matches.
name: preg_split
order: 3
typeParameters: []
parameters:
  - name: pattern
    type: string
    description: UTF-8 PCRE2 pattern delimited by /, #, or ~.
  - name: subject
    type: string
    description: Binary subject.
  - name: limit
    type: int
    description: Maximum split parts; -1 is unlimited and 1 returns the unsplit subject.
    default: "-1"
  - name: flags
    type: int
    description: Constant bit mask, 1 omits empty parts, 2 includes delimiter captures, and 4 returns offset captures.
    default: "0"
returns:
  type: "vector<string>"
  description: Split parts; flag 4 changes the element type to RegexCapture.
errors:
  - description: Invalid patterns, modifiers, limits below -1, or PCRE2 match failures are runtime errors. Nonconstant or unsupported flags are compile errors.
related: []
status: experimental
availability: implemented
notice: The result type depends on constant flags. Split limits count delimiter matches; included capture parts do not consume the limit.
version: "0.11"
module: extensions
---

`preg_split()` returns text between matches. Flag `1`
(`PREG_SPLIT_NO_EMPTY`) omits empty parts, flag `2`
(`PREG_SPLIT_DELIM_CAPTURE`) inserts participating groups, and flag `4`
(`PREG_SPLIT_OFFSET_CAPTURE`) returns
[`RegexCapture`](thp:std.extensions.RegexCapture) objects with byte offsets.
The flags must be a non-negative integer constant.

```thp
$parts = preg_split('/,/', 'a,b');
echo $parts[1]; // b
```
