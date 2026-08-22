---
kind: guide
id: guide.languageExceptions
title: Exceptions
summary: Describes throwing, catching, and propagating THP failures.
nav:
  section: language
  order: 140
status: experimental
availability: partial
notice: >-
  Throwable subtyping, ordered catches, propagation, `finally`, previous
  chains, and suppressed `using` cleanup failures are experimental. Multi-catch
  syntax and compile-time checked-exception analysis remain out of scope.
---

`throw` raises a `Throwable` value. Only `Exception`, `Error`, and their
descendants are throwable. `Throwable` is sealed: an ordinary class cannot
implement it directly.

The following example uses the proposed `ValueError` public spelling:

```thp
try {
    configurePort($port);
} catch (ValueError $error) {
    echo $error->getMessage();
}
```

## Catching

A catch parameter must name a `Throwable` subtype. Execution selects the
innermost active handler and tests that handler's clauses in source order.
Duplicate catches and catches already subsumed by an earlier clause are compile
errors.

THP distinguishes application-defined exceptional conditions from failures
detected by the language or runtime. The hierarchy roots those branches at
`Exception` and `Error`, both under `Throwable`. User subclasses participate in
the same nominal matching and runtime state as native exceptions.

`Exception` provides:

```thp
public function __construct(
    string $message = "",
    int $code = 0,
    ?Throwable $previous = null,
);
public function getMessage(): string;
public function getCode(): int;
public function getPrevious(): ?Throwable;
public function getSuppressed(): vector<Throwable>;
```

## Propagation

A function that does not catch a failure passes it to its caller. Rethrowing
preserves the same throwable object.

`try` accepts catches, `finally`, or both:

```thp
try {
    performWork();
} catch (Exception $error) {
    report($error);
} finally {
    releaseLease();
}
```

`finally` runs after normal completion, after a selected catch, and before a
pending `return`, `break`, `continue`, or throwable resumes. A return expression
is evaluated before `finally`. A transfer initiated by `finally` replaces the
pending transfer.

When a throwable from `finally` replaces a pending throwable, the pending
throwable is appended to the end of the replacement's existing `previous`
chain. Exceptions raised by a catch or `finally` are visible only to enclosing
regions. Nested cleanup runs innermost first and a `finally` body never
re-enters itself.

Cleanup failures that happen while another exception is already propagating
are appended to the primary throwable's suppressed exceptions. This prevents a
`using` cleanup failure from hiding the failure that caused the block to exit.
Suppression is distinct from `finally` replacement: `using` retains its primary
failure and records cleanup failures separately.

Bytecode verification failures, internal VM failures, instruction-limit
failures, and execution-time aborts are host/runtime failures rather than
source throwables. They bypass catches and source cleanup.

## See also

- [Errors](thp:guide.languageErrors)
- Predefined exceptions
- `Throwable`
- [Resources and streams](thp:guide.languageResourcesAndStreams)
