---
kind: class
id: std.spl.RegexIterator
title: RegexIterator
summary: Filters or transforms iterator values using a regular expression.
name: RegexIterator
module: iterators
typeParameters:
  - name: K
    description: The key type preserved from the wrapped iterator.
  - name: TIn
    description: Input value type; string is required except in USE_KEY mode.
  - name: TOut
    description: Output value type fixed by the constant mode and flags.
interfaces:
  - id: std.baseTypes.Iterator
    arguments:
      - K
      - TOut
constants:
  - name: MATCH
    type: int
    description: Filters by a regular-expression match.
  - name: GET_MATCH
    type: int
    description: Yields the first match details.
  - name: ALL_MATCHES
    type: int
    description: Yields all match details.
  - name: SPLIT
    type: int
    description: Yields regular-expression splits.
  - name: REPLACE
    type: int
    description: Yields replacement results.
  - name: USE_KEY
    type: int
    description: Filters on string keys while preserving input values.
properties: []
status: experimental
availability: implemented
notice: The VM implements fixed modes and flags. The class is not a PHP inheritance or behavior compatibility promise.
version: "0.11"
---

`RegexIterator<K, TIn, TOut>` filters or transforms iterator values using a
regular expression.

## Construction

| Method                                                    | Description                                                               |
| --------------------------------------------------------- | ------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.RegexIterator::__construct) | Wraps the input iterator and fixes the output type for the selected mode. |

## Behavior

The constructor takes `Iterator<K, TIn>`, a delimited PCRE2 pattern, constant
mode, optional inversion, constant flags, and an optional replacement template.
Construction validates the pattern. The iterator searches lazily, preserves
keys, and compiles results to `Iterator<K, TOut>`.

| Mode                | Value type `TOut`                                    | Behavior                                          |
| ------------------- | ---------------------------------------------------- | ------------------------------------------------- |
| `MATCH` (`0`)       | `TIn`                                                | Keep matching string values.                      |
| `GET_MATCH` (`1`)   | `map<string, string>` or offset capture map          | Yield first capture map.                          |
| `ALL_MATCHES` (`2`) | `vector<map<string, string>>` or offset capture maps | Yield all capture maps.                           |
| `SPLIT` (`3`)       | `vector<string>` or `vector<RegexCapture>`           | Split each selected value.                        |
| `REPLACE` (`4`)     | `string`                                             | Replace matches using the constructor's template. |
| `USE_KEY` (`5`)     | `TIn`                                                | Match string keys and preserve values.            |

Modes other than `USE_KEY` require `TIn = string`. `USE_KEY` requires `K =
string`. The mode and flags must be integer constants because they fix `TOut`.
`GET_MATCH` and `ALL_MATCHES` accept match flags `256` and `512`; `SPLIT`
accepts split flags `1`, `2`, and `4`. `MATCH`, `REPLACE`, and `USE_KEY` accept
zero flags. `$invert` selects inputs that do not match. An inverted capture
mode yields an empty capture result for a selected nonmatch.

## Differences from PHP

THP selects inverted matching with a boolean and checks mode and flags at
compile time. Its output types and mode values are defined by this contract.

## Errors

Invalid patterns fail at construction. Cursor operations propagate failures
from the wrapped iterator or PCRE2 engine at the cursor call site.

## Example

```thp
$thpFiles = new RegexIterator(new VectorIterator(['a.thp', 'b.txt']), '/[.]thp$/');
```

Only matching path strings are yielded in the default mode.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `RegexIterator`](https://www.php.net/manual/en/class.regexiterator.php)
