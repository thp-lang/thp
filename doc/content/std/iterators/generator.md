---
kind: class
id: std.spl.Generator
title: Generator
summary: A one-shot, lazily resumed iterator created by a function containing yield.
name: Generator
module: iterators
typeParameters:
  - name: K
    description: Type of yielded keys.
  - name: V
    description: Type of yielded values.
interfaces:
  - id: std.baseTypes.Iterator
    arguments:
      - K
      - V
constants: []
properties: []
status: experimental
availability: implemented
notice: Generator functions, cursor methods, getReturn(), and close() execute in the standalone VM.
version: "0.7"
---

`Generator<K, V>` is created by calling a function that contains a `yield`
statement and declares `Generator<K, V>` or `Iterator<K, V>`. It cannot be
constructed directly. It implements the ordinary
[`Iterator<K, V>`](thp:std.baseTypes.Iterator) cursor protocol and is one-shot.

## Methods

| Method                                            | Description                                                    |
| ------------------------------------------------- | -------------------------------------------------------------- |
| [`getReturn()`](thp:std.spl.Generator::getReturn) | Reads the final result after normal exhaustion.                |
| [`close()`](thp:std.spl.Generator::close)         | Unwinds a suspended frame and closes active `using` resources. |

The cursor methods `rewind()`, `valid()`, `key()`, `value()`, and `advance()`
follow the [`Iterator`](thp:std.baseTypes.Iterator) contract. The first
`rewind()` starts the body. After advancement, a later `rewind()` throws
`Error`. `key()` and `value()` fail without a suspended pair.

See [Generators](thp:guide.languageGenerators) for key assignment, return
values, exceptions, and cleanup rules.
