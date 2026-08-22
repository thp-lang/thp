---
kind: class
id: std.async.Timeout
title: Timeout
summary: Represents a one-shot timer used to limit an asynchronous wait.
name: Timeout
module: async
typeParameters: []
interfaces:
  - id: std.async.Completable
    arguments:
      - mixed
constants: []
properties: []
status: experimental
availability: proposed
notice:
  Timeout is proposed and is not implemented in this repository. Its failure type
  and cancellation behavior are not finalized.
version: "0.1"
---

This is a final class.

`Timeout` is the one-shot timer returned by [`timeout()`](thp:std.async.timeout). It can
be supplied as the cancellation source for [`await()`](thp:std.async.await).

## Construction

`Timeout` has no public constructor. Create one with [`timeout()`](thp:std.async.timeout).

## Behavior

A fired timeout completes with a failure rather than a value. Its
`Completable<mixed>` parameter exists only so it can serve as a general
cancellation source; successful completion does not produce a value. When used
as the second argument of `await()`, timeout completion cancels only the wait;
the target operation remains active.

## Example

```thp
$limit: Timeout = timeout(5_000);

if (cacheHit()) {
    $limit->cancel();
}
```

## See also

- [`timeout()`](thp:std.async.timeout)
- [`await()`](thp:std.async.await)
- [`Completable<T>`](thp:std.async.Completable)
- [True Async Timeout](https://true-async.github.io/en/docs/reference/timeout.html)
