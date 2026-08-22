---
kind: guide
id: guide.languageTypes
title: Types
summary: Describes THP's scalar, compound, nullable, union, and generic types.
nav:
  section: language
  order: 30
status: experimental
availability: partial
notice: >-
  Scalars, nullable and union types, user classes, native vectors and maps, and
  arbitrary-byte strings execute. Generic user types and the broader proposed
  type system remain unimplemented.
---

THP checks declared types before and during execution. THP replaces PHP's
general-purpose `array` with native generic `vector<T>` and `map<K, V>` types.
They keep familiar bracket access, iteration, and collection functions while
using distinct literal syntax.

## Core types

| Type        | Meaning                                                   |
| ----------- | --------------------------------------------------------- |
| `int`       | Signed 64-bit integer.                                    |
| `float`     | Double-precision floating-point number.                   |
| `bool`      | `true` or `false`.                                        |
| `string`    | Arbitrary byte string.                                    |
| `null`      | The null value.                                           |
| `object`    | An object value.                                          |
| `callable`  | A callable value.                                         |
| `mixed`     | A value whose more precise type is unknown.               |
| `void`      | A callable that returns no useful value.                  |
| `never`     | A callable that does not return normally.                 |
| `vector<T>` | Ordered native collection containing values of type `T`.  |
| `map<K, V>` | Native collection mapping keys `K` to values of type `V`. |

## Collections

`vector<T>` stores an ordered sequence of values of type `T`.
`map<K, V>` stores insertion-ordered key/value pairs.

```thp
$names: vector<string> = [];
array_push($names, "Ada");

$scores: map<string, int> = {"Ada" => 10};
$scores["Grace"] = 9;
```

Square brackets create a vector. Vector literals contain unkeyed values:

```thp
$emptyNames: vector<string> = [];
$names = ["Ada", "Grace"];
```

Braces create a map, and `=>` separates each key from its value:

```thp
$emptyScores: map<string, int> = {};
$scores = {"Ada" => 10, "Grace" => 9};
```

A non-empty literal infers its generic types from its entries. An empty literal
uses its declared or otherwise expected type when its generic arguments cannot
be inferred.

Keyed entries are not valid inside `[]`, and unkeyed entries are not valid
inside `{}`. Both collection types use bracket access, variable-rooted element
assignment, and direct `foreach` traversal. A vector index must be an integer;
a map key must be compatible with `K`. Their generic arguments let THP reject
incorrect indices, keys, and values.

Map traversal follows insertion order. Keyed `foreach` binds the map's `K` and
`V` directly; vector traversal binds an `int` offset and `T` value. Iterator
objects are not yet accepted by the executable `foreach` implementation.
The proposed object protocol represents both shapes as `Iterator<K, V>`;
vector iterators use `int` for `K`, while map iterators preserve their declared
key type.

Vectors and maps are native values rather than ordinary class instances. This
allows the compiler and VM to lower common collection operations directly
without requiring an object wrapper or dynamic method dispatch. Their internal
storage remains an implementation detail. Collection values use copy-on-write:
mutating an alias detaches its storage and does not modify earlier copies.

## Composite type declarations

Generic arguments use angle brackets. `?T` is shorthand for a nullable type,
and `A|B` accepts either member.

```thp
function findName(map<int, string> $names, int $id): ?string {
    return $names[$id] ?? null;
}
```

Classes, interfaces, and enumerations define user types. Their advanced generic
and runtime behavior is still evolving.

## Type conversion

Casts, strict equality, and declared-type checks are part of the tested
language surface. THP does not promise PHP's full type-juggling rules; code
should use explicit conversions when a value changes type.

Output is a narrow exception rather than a general conversion to `string`.
`echo` and string concatenation accept `string`, `int`, `float`, and `bool` and
apply one canonical, locale-independent output conversion. They reject `null`
and any union containing `null` until code narrows the value or supplies an
explicit fallback. In particular, output conversion does not make an `int`
assignable to a `string` variable.

## Strings and text

`string` is a sequence of arbitrary bytes. Its length, offsets, comparisons,
and concatenation operate on bytes, matching PHP strings and allowing binary
stream I/O without validation or transcoding.

APIs that interpret a string as UTF-8 validate it explicitly. Invalid UTF-8
raises `ValueError` unless the API documents a replacement policy. This keeps
binary data lossless while making text assumptions visible at the operation
that needs them.

## See also

- [Variables](thp:guide.languageVariables)
- [Functions](thp:guide.languageFunctions)
- [Classes and objects](thp:guide.languageClassesAndObjects)
- [Enumerations](thp:guide.languageEnumerations)
- [Resources and streams](thp:guide.languageResourcesAndStreams)
