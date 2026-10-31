---
kind: function
id: std.extensions.preg_match
title: preg_match
summary: Matches a PCRE2 pattern and returns typed captures.
name: preg_match
order: 1
typeParameters: []
parameters:
  - name: pattern
    type: string
    description: UTF-8 PCRE2 pattern delimited by /, #, or ~.
  - name: subject
    type: string
    description: Binary subject to search.
  - name: flags
    type: int
    description: Constant bit mask, 256 for offsets and 512 for unmatched nulls.
    default: "0"
returns:
  type: "map<string, string>|null"
  description: Null on no match; a map of numeric and named captures on success. Flag 256 changes values to RegexCapture; flag 512 permits null values.
errors:
  - description: Invalid patterns, modifiers, or PCRE2 match failures are runtime errors at the call site. Nonconstant or unsupported flags are compile errors.
related: []
status: experimental
availability: implemented
notice: The result type depends on constant flags; runtime autoload and PHP by-reference output arguments are not part of this API.
version: "0.11"
module: extensions
---

`preg_match()` searches once. Capture keys are decimal group numbers and declared
group names. Unmatched groups contain an empty string by default. Flag `512`
(`PREG_UNMATCHED_AS_NULL`) returns null for an unmatched group. Flag `256`
(`PREG_OFFSET_CAPTURE`) returns [`RegexCapture`](thp:std.extensions.RegexCapture)
objects; an unmatched capture has null text and offset `-1`. Offsets count bytes.
The flags must be a non-negative integer constant because they change the
static result type.

```thp
$match = preg_match('/(?<word>thp)/', 'hello thp', 256);
if (is_map($match)) {
    echo $match['word']->offset();
}
```

Only `i`, `m`, `s`, `x`, and `u` pattern modifiers are accepted. `u` enables
UTF-8 validation and Unicode properties. Other modifiers and malformed
delimiters fail at runtime.
