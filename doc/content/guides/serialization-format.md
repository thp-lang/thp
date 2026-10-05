---
kind: guide
id: guide.serializationFormat
title: THP serialization format
summary: Version-one binary encoding for THP scalar and collection values.
nav:
  section: learn
  order: 16
status: experimental
availability: implemented
notice: >-
  The reference VM implements this format for scalars, vectors, and maps.
  Object serialization hooks and PHP serialization compatibility remain proposed.
---

`serialize(mixed $value): string` and `unserialize(string $value): mixed` use
THP's own binary format. Runtime strings are arbitrary bytes, and serialized
strings can contain NUL and non-UTF-8 bytes. The format is not PHP's
`serialize()` format.

## Accepted values

The version-one format accepts `null`, `bool`, signed 64-bit `int`, IEEE-754
64-bit `float`, binary `string`, `vector<T>`, and insertion-ordered
`map<K, V>`. Map keys must be `int` or `string`. Nested values follow the same
rule. Objects, exceptions, closures, generators, streams, and cyclic values
are rejected. `Option<T>` is an object and is not serializable in version one.

The encoder preserves scalar bits, vector order, map insertion order, and
distinct integer and string map keys. It does not store generic type arguments.
The decoder returns `mixed`; decoded vectors use `mixed` elements, and maps use
`int|string` keys with `mixed` values. Use `is_vector()` or `is_map()` to narrow a decoded value before
collection operations. No constructor, destructor, or user method runs during
decoding.

## Byte layout

All lengths are unsigned 32-bit little-endian integers. A document starts with
the five bytes `54 48 50 53 01` (`THPS` and format version `1`), followed by
exactly one value. Values use these tags:

| Tag  | Value    | Payload                                              |
| ---- | -------- | ---------------------------------------------------- |
| `00` | `null`   | none                                                 |
| `01` | `false`  | none                                                 |
| `02` | `true`   | none                                                 |
| `03` | `int`    | eight little-endian two's-complement bytes           |
| `04` | `float`  | eight little-endian IEEE-754 bits                    |
| `05` | `string` | byte length, then exact bytes                        |
| `06` | `vector` | element count, then values in order                  |
| `07` | `map`    | entry count, then key/value pairs in insertion order |

Decoders reject unknown tags, invalid map keys, duplicate keys, truncated
payloads, and trailing bytes. The document is limited to 16 MiB including its
header; a value may have at most 64 recursive value edges and one million total
values. These limits also
bound cyclic graphs, which cannot be encoded by this tree format.

## Failures

`serialize()` throws `InvalidArgumentException` for unsupported values or
format limits. `unserialize()` throws `UnexpectedValueException` for malformed
or oversized input. Allocation failures propagate as runtime errors. The
diagnostic location is the call expression. Neither API mutates its input;
decoding creates request-owned values with ordinary collection cleanup.

```thp
$bytes = serialize([1, 2, 3]);
$decoded = unserialize($bytes);
if (is_vector($decoded)) {
    echo count($decoded);
}
```
