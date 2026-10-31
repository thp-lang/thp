---
kind: function
id: std.async.async
title: async
summary: Creates and schedules a coroutine for concurrent execution.
name: async
order: 10
typeParameters:
  - name: T
    description: The T type parameter.
parameters:
  - name: function
    type: callable
    description: The callable executed by the coroutine.
  - name: arguments
    type: mixed
    description: Values passed to the callable, in order.
    variadic: true
returns:
  type: Coroutine<T>
  description:
    The newly created Coroutine, where T is the callable's return type. Pass
    this handle to await() to retrieve the result.
errors:
  - description: An unregistered scheduler throws Error at this call site.
  - description: Incompatible callable arguments are compile errors.
related: []
status: experimental
availability: implemented
version: "0.1"
module: async
---

`async()` schedules a callable to run in a new coroutine and immediately returns
its typed handle. It is THP's spelling of the operation called `spawn()` in True
Async.

## Behavior

`async()` requires a statically typed callable. The compiler checks each
argument against its parameter and infers `T` from the callable's return type.
THP enqueues its body and returns `Coroutine<T>` before that body runs.

The callable does not start before `async()` returns. Its result or
failure is stored by the coroutine runtime:

- returning from `$function` completes the coroutine successfully;
- throwing from `$function` completes it with the same failure; and
- [`await()`](thp:std.async.await) retrieves the value or rethrows the failure.

Starting several coroutines before awaiting them allows their work to overlap at
cooperative suspension points.

Direct file-backed opens, reads, writes, seeks, EOF checks, flushes,
`SplFileObject` line methods, `SplFileInfo` metadata queries, directory
iteration, and temporary-file spill suspend while a worker performs host I/O.
Memory-stream operations complete immediately. `thp:/input`, host output, and
ABI-v1 native callbacks may block the request thread.

## Example

```thp
<?thp

scheduler_register_default();

$first: Coroutine<string> = async(fetchText(...), "/first");
$second: Coroutine<string> = async(fetchText(...), "/second");

echo await($first);
echo await($second);
```

The compiler rejects an argument whose type does not match the callable's
parameter. It does not defer that error until the body runs.

The two operations are scheduled before either result is requested. This mirrors
True Async's model of launching a callable and returning a handle for its
eventual result while preserving THP's static result type.

## See also

- [`Coroutine<T>`](thp:std.async.Coroutine)
- [`await()`](thp:std.async.await)
- [`suspend()`](thp:std.async.suspend)
- [`delay()`](thp:std.async.delay)
- [`scheduler_register_default()`](thp:std.async.scheduler_register_default)
- [`scheduler_register()`](thp:std.async.scheduler_register)
- [`scheduler_is_registered()`](thp:std.async.scheduler_is_registered)
- [`SchedulerInterface`](thp:std.async.SchedulerInterface)
- [True Async `spawn()`](https://true-async.github.io/en/docs/reference/spawn.html)
