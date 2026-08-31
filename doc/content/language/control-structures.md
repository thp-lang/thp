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

Executable `foreach` evaluates its source once and accepts native `vector<T>`
and insertion-ordered `map<K, V>` values. The proposed object form additionally
accepts every `Traversable<K, V>` object; ordinary object properties are never
an implicit traversal source.

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
Native traversal captures the source collection's COW snapshot. Mutating an
alias in the body detaches that alias and does not change the elements selected
by the active loop.
The plain form binds only the value:

```thp
foreach ($values as $value) {
    echo $value . "";
}
```

New key and value variables are scoped to the loop. A compatible existing
local is reused and retains its final assigned value. Reusing an incompatible
local, using the same variable for key and value, or iterating a string or
scalar is a compile error. In the proposed object form, loop bindings are the
invariant `K` and `V` from `Traversable<K, V>`. Keys remain strict THP values;
there is no PHP array-key coercion.

### Proposed object traversal protocol

Iterator-object traversal is not implemented. Its contract is:

1. Evaluate the `foreach` source exactly once.
2. For an `IteratorAggregate<K, V>`, call `getIterator()` exactly once at that
   layer. Repeat only if the returned `Traversable<K, V>` is another aggregate.
3. For the resulting direct `Iterator<K, V>`, call `rewind()` once.
4. Repeat `valid() → value() → optional key() → body → advance()`.

`value()` is called once per selected iteration and `key()` only for the keyed
form. `continue` reaches `advance()`. `break`, `return`, and a throw leave
without advancing. Iterator throwables propagate unchanged, while transfers
still honor enclosing `using` and `finally` regions. A one-shot iterator or
generator may reject `rewind()` after it has advanced, so a second traversal
of the same cursor may fail.

A direct iterator owns its cursor:

```thp
<?thp

class PairIterator implements Iterator<string, int>
{
    private int $position = -1;

    public function rewind(): void { $this->position = 0; }
    public function valid(): bool { return $this->position < 2; }
    public function key(): string { return $this->position === 0 ? "left" : "right"; }
    public function value(): int { return $this->position + 10; }
    public function advance(): void { $this->position = $this->position + 1; }
}

foreach (new PairIterator() as $key => $value) {
    echo $key . "=" . $value . "\n";
}
```

An aggregate delegates each layer exactly once and may return either strategy:

```thp
<?thp

class Pairs implements IteratorAggregate<string, int>
{
    public function getIterator(): Traversable<string, int>
    {
        return new PairIterator();
    }
}
```

A concrete class may not implement `Traversable<K, V>` directly or implement
both strategies. An abstract class may defer the choice to its concrete
subclass:

```thp
<?thp

class InvalidDirect implements Traversable<int, string> {}

class InvalidDual
    implements Iterator<int, string>, IteratorAggregate<int, string>
{
    // Compile error regardless of the methods supplied.
}
```

One-shot rewind failure is observable:

```thp
<?thp

$lines = readLinesOnce();
foreach ($lines as $line) { break; }
foreach ($lines as $line) { echo $line; } // rewind() may throw unchanged.
```

Mutation of a delegated iterator object remains visible according to its
methods, unlike native collection snapshot traversal:

```thp
<?thp

$iterator = new MutableIterator<int, string>(["first"]);
foreach ($iterator as $value) {
    $iterator->append("later"); // May be observed by a later valid()/value().
}
```

Cleanup ordering is preserved when an iterator operation throws:

```thp
<?thp

try {
    using ($stream = MemoryStream::open()) {
        foreach (new ThrowingIterator() as $value) {
            echo $value;
        }
    } // close() runs while the iterator throwable remains primary.
} finally {
    echo "finally\n";
}
```

The iterator throwable remains the pending primary failure while `close()` and
the outer `finally` run, then propagates unchanged. If cleanup also fails, that
failure is suppressed on the iterator throwable.

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
