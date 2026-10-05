---
kind: guide
id: guide.languageGenerators
title: Generators
summary: Defines one-shot generator functions and their Iterator cursor behavior.
nav:
  section: language
  order: 150
status: experimental
availability: implemented
notice: >-
  Statement-form yield, generator cursors, explicit close, and final return values execute in the VM. The baseline JIT uses the VM fallback in automatic mode. Send operations and yield expressions are not supported.
---

A function or block closure containing `yield` is a generator. Its declared
return type must be `Generator<K, V>` or `Iterator<K, V>`. Calling it evaluates
arguments and captures, then creates a generator without executing its body.
The generator implements the existing [`Iterator<K, V>`](thp:std.baseTypes.Iterator)
cursor protocol. There is no second traversal protocol.

```thp
<?thp
function entries(): Generator<int, string> {
    yield "first";
    yield 8 => "second";
    yield "third";
    return 42;
}

$entries = entries();
foreach ($entries as $key => $value) {
    echo $key . ":" . $value . "\n";
}
var_dump($entries->getReturn()); // int(42)
```

## Syntax and typing

`yield $value;` and `yield $key => $value;` are statements. The key and value
expressions are evaluated once, left to right, immediately before suspension.
The value must be assignable to `V`. An explicit key must be assignable to `K`.
An omitted key is an `int`, so `K` must accept `int`. A `yield` outside a
generator function, a generator with another declared return type, and a
`yield` expression are compile errors. `yield` has no result slot and callers
cannot send values into a suspended generator.

Automatic keys start at zero. Each automatic key consumes the next integer.
An explicit integer key at or above that next integer moves the following
automatic key to the explicit key plus one; a smaller integer or a noninteger
key leaves the counter alone. No PHP key coercion occurs. If a later automatic
key would exceed signed 64-bit range, that resume throws `ValueError`.

`return $result;` stores any THP value as the generator's final result; an
implicit end or `return;` stores `null`. The declared `Generator<K, V>` type
describes yielded pairs, not this result. After normal exhaustion,
`getReturn(): mixed` retrieves it. Before exhaustion or after explicit close,
`getReturn()` throws `Error`.

## Cursor lifecycle

The first `rewind()` executes the body until the first yield or completion.
Repeated `rewind()` calls before any `advance()` leave the first pair in place.
After advancement, `rewind()` throws `Error`, including after exhaustion.
`valid()` reports whether a pair is suspended; `key()` and `value()` return it
without resuming. `advance()` resumes until the next yield or completion. It
has no effect on a fresh, exhausted, or closed generator. An exhausted
generator remains exhausted. `key()` and `value()` fail outside a suspended
pair. A second `foreach` over an advanced generator fails at its initial
`rewind()`.

A throw in the body occurs on the `rewind()` or `advance()` that reaches it,
including before the first yield. Existing `catch`, `finally`, and `using`
semantics apply while the frame runs. An uncaught exception propagates
unchanged and exhausts the generator; it has no final return value.

## Suspension and cleanup

Suspension retains locals, temporary values, exception handlers, and active
`using` and `finally` scopes. Merely suspending does not run cleanup. Normal
resumption or return runs those scopes in their usual order. `close()` on a
fresh generator skips the body and marks it closed. On a suspended generator,
`close()` unwinds active scopes: inner `using` resources close before outer
`finally` blocks. User catches cannot intercept the internal close signal.
Cleanup failures propagate from `close()`; multiple `using` failures retain
the later ones as suppressed exceptions. `close()` is idempotent. Calling it
after normal exhaustion preserves the final result.

After explicit close, `valid()` is false, `advance()` has no effect,
`rewind()` throws `Error`, and `getReturn()` has no result. Yielding while a
close is unwinding throws `Error`. Dropping the final reference to a suspended
generator releases its frame and owned values. Explicit `close()` is required
to run user `finally` code and `Closeable::close()` deterministically.

## See also

- [`Generator<K, V>`](thp:std.spl.Generator)
- [Control structures](thp:guide.languageControlStructures)
- [`Iterator<K, V>`](thp:std.baseTypes.Iterator)
