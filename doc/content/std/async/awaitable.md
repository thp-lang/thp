---
kind: interface
id: std.async.Awaitable
title: Awaitable
summary: Marks an asynchronous value whose readiness can be observed by a scheduler.
name: Awaitable
module: async
typeParameters:
  - name: T
    description: The value type produced when the source is ready.
interfaces: []
constants: []
properties: []
status: experimental
availability: proposed
notice: Awaitable is a proposed generic interface and is not implemented in this repository.
version: "0.1"
---

`Awaitable<T>` marks an asynchronous source that can produce values of type
`T`.

## Contract

`Awaitable` is a marker interface. The scheduler and higher-level asynchronous
operations use it to recognize readiness sources; it does not expose a public
polling method.

An awaitable may become ready more than once. One-shot results use the stronger
[`Completable<T>`](thp:std.async.Completable) contract.

The generic parameter records the produced value type even though the marker
interface has no methods. This lets APIs preserve result types rather than
falling back to `mixed`.

## Implementations

| Type                                          | Notes                                     |
| --------------------------------------------- | ----------------------------------------- |
| [`Completable<T>`](thp:std.async.Completable) | A result that reaches one terminal state. |

Future channel and stream designs may implement `Awaitable<T>` directly when
one object can become ready repeatedly.

## Example

An integration can accept any readiness source without requiring it to be
one-shot:

```thp
function watchReadiness<T>(Awaitable<T> $source): void {
    registerReadiness($source);
}
```

## See also

- [`Completable<T>`](thp:std.async.Completable)
- [`Coroutine<T>`](thp:std.async.Coroutine)
- [`await()`](thp:std.async.await)
- [True Async base interfaces](https://true-async.github.io/en/docs/components/interfaces.html)
