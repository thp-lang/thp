---
kind: function
id: std.extensions.preg_replace
title: preg_replace
summary: Replaces PCRE2 matches in a binary string.
name: preg_replace
order: 2
typeParameters: []
parameters:
  - name: pattern
    type: string
    description: UTF-8 PCRE2 pattern delimited by /, #, or ~.
  - name: replacement
    type: string
    description: Replacement template using $0, $1, named ${group}, and $$.
  - name: subject
    type: string
    description: Binary subject.
  - name: limit
    type: int
    description: Maximum replacements; -1 is unlimited and 0 leaves the subject unchanged.
    default: "-1"
returns:
  type: string
  description: Binary string after replacement.
errors:
  - description: Invalid patterns, modifiers, limits below -1, or PCRE2 match failures are runtime errors at the call site.
related: []
status: experimental
availability: implemented
notice: The replacement template supports the documented THP subset and is not a promise of PHP replacement syntax compatibility.
version: "0.11"
module: extensions
---

`preg_replace()` replaces at most `$limit` matches from left to right. A missing
capture expands to an empty byte string.

```thp
echo preg_replace('/(a)/', '$1!', 'banana'); // ba!na!na!
```
