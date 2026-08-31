---
kind: guide
id: guide.languageGenerators
title: Generators
summary: Describes the proposed THP model for resumable sequence producers.
nav:
  section: language
  order: 150
status: experimental
availability: proposed
notice: >-
  Generator syntax and runtime behavior are proposed and are not yet backed by an implementation in this checkout.
---

A generator is a function that can suspend after producing a value and later
resume from the same point. THP reserves PHP-shaped `yield` syntax for this
model.

```thp
function integers(int $limit): Traversable<int, int> {
    for ($index = 0; $index < $limit; $index = $index + 1) {
        yield $index;
    }
}
```

Generators use the invariant `Traversable<K, V>` protocol and may be one-shot:
the initial `foreach` rewind succeeds, but rewinding after advancement may
throw. Yielded keys retain strict THP values without PHP key coercion.
Sent values, return values, explicit closure, and the concrete failure types
remain unresolved. None of those details should yet be treated as executable.

Future lazy transformations will use iterator adapters. Eager native collection
operations use shape-prefixed names such as `vector_map()` and `map_filter()`.
Neither family is implemented in this checkout.

## See also

- [Control structures](thp:guide.languageControlStructures)
- [`Traversable<K, V>`](thp:std.baseTypes.Traversable)
- [Iterators](thp:std.iterators)
