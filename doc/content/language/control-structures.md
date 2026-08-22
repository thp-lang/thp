---
kind: guide
id: guide.languageControlStructures
title: Control structures
summary: Describes THP branches, loops, matching, and control transfers.
nav:
  section: language
  order: 80
status: experimental
availability: partial
notice: >-
  The compiler and reference VM implement the forms described here. Their
  syntax and diagnostics remain experimental; iterator objects, numeric
  `break`/`continue` levels, and multi-catch syntax are not implemented.
---

THP uses brace-delimited control structures.

## Branching

`if`, `elseif`, and `else` select statements by boolean conditions.

```thp
if ($score >= 90) {
    echo "excellent";
} elseif ($score >= 60) {
    echo "pass";
} else {
    echo "retry";
}
```

`match` evaluates its subject exactly once. It tests arm conditions from left
to right with strict equality and evaluates only the result of the first
matching arm. Comma-separated conditions share a result, and a trailing comma
is accepted.

```thp
$label = match ($status) {
    200, 201 => "success",
    404 => "missing",
    default => "other",
};
```

Conditions are full typed expressions and must have a type that can overlap
the subject type. Duplicate literal conditions and multiple `default` arms are
compile errors. The result type is the normalized union of reachable arm
result types, constrained by an expected type when one is available.

If no condition matches and there is no `default`, evaluation throws
[`UnhandledMatchError`](thp:std.baseTypes.UnhandledMatchError). Its message
contains a bounded deterministic description of the unmatched subject. The
error is catchable as either `UnhandledMatchError` or
[`Error`](thp:std.baseTypes.Error).

## Loops

`while`, `for`, and `foreach` are part of the tested surface.

`for` accepts empty or comma-separated initializer, condition, and update
clauses. Clauses run left to right. Initializers run once, all condition
expressions run before each iteration and the last condition controls entry,
and updates run after the body and after `continue`. An empty condition is
`true`.

```thp
for ($left: int = 0, $right: int = 5;
     $left < 100, $left < $right;
     $left = $left + 1, $right = $right - 1) {
    echo $left . "\n";
}
```

`foreach` evaluates its source once and currently accepts only native
`vector<T>` and insertion-ordered `map<K, V>` values.

```thp
foreach ($values as $key => $value) {
    if ($value < 0) {
        continue;
    }

    echo $key . "=" . $value . "\n";
}
```

For a vector, the key type is `int` and keys are zero-based offsets. For a map,
keys and values retain `K` and `V` and traversal follows insertion order.
The plain form binds only the value:

```thp
foreach ($values as $value) {
    echo $value . "";
}
```

New key and value variables are scoped to the loop. A compatible existing
local is reused and retains its final assigned value. Reusing an incompatible
local, using the same variable for key and value, or iterating a string,
object, or scalar is a compile error. Iterator-object traversal remains
unimplemented. Its proposed lowering calls `rewind()` once, checks `valid()`
before each iteration, reads `value()` and, only for a keyed loop, `key()`, then
calls `advance()`. All five operations belong to the same `Iterator<K, V>`
interface, so `foreach` does not test for a separate rewindable capability.

`break` exits the innermost loop and `continue` starts its next iteration.
Both are rejected outside a loop. Only level-one `break;` and `continue;` are
accepted; numeric levels are diagnosed during parsing. Transfers leaving a
`using` or `finally` region still run its cleanup. These loop forms use direct
VM collection operations; execution modes that cannot compile them fall back
to the reference VM.

## Transfers and failures

`return` completes a function. `throw` transfers control to a compatible
`catch`. `finally` runs before fallthrough or any pending `return`, `break`,
`continue`, or throwable resumes; a transfer from `finally` replaces the
pending transfer.

`using` requires a nominal
[`Closeable`](/language/resources-and-streams/#deterministic-cleanup) value on every exit path:

```thp
using ($stream = MemoryStream::open()) {
    $stream->writeAll("temporary data");
}
```

The binding is scoped to the block. If its body and cleanup both throw, the
body failure remains primary and the cleanup failure is attached as
suppressed.

THP does not document PHP's alternative control-structure syntax or `goto` as
supported executable features.

## See also

- [Expressions](thp:guide.languageExpressions)
- [Functions](thp:guide.languageFunctions)
- [Exceptions](thp:guide.languageExceptions)
- [Resources and streams](thp:guide.languageResourcesAndStreams)
