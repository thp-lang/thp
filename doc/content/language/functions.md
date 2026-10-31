---
kind: guide
id: guide.languageFunctions
title: Functions
summary: Describes typed THP function declarations, parameters, returns, and callables.
nav:
  section: language
  order: 90
status: experimental
availability: partial
notice: >-
  Typed declarations, statically resolved calls, constant defaults, named
  arguments, variadic parameters, typed callable values, closures, and callable
  invocation execute. Generic functions, by-reference parameters, call-site
  unpacking, and arbitrary dynamic function names remain proposals.
---

Functions declare parameter types before variable names and a return type after
the parameter list.

```thp
function double(int $value): int {
    return $value * 2;
}
```

## Parameters and arguments

Each argument must satisfy its parameter's declared type. Generic functions
introduce type parameters after the function name.

```thp
function first<T>(vector<T> $values): ?T {
    return $values[0] ?? null;
}
```

## Default arguments

A parameter may declare a typed constant default. Supported defaults are
scalar and `null` literals, unary constant expressions, and recursively
constant vector and map literals.
Typed constant defaults may nest at most 128 vector or map levels.

```thp
function retry(
    int $attempts = 3,
    vector<int> $delays = [1, 2, 5],
): int {
    return $attempts + count($delays);
}
```

Defaults are type-checked at their declarations and materialized for each
omitted argument, so mutable collection defaults are not shared between
calls. A required parameter cannot follow a defaulted parameter.
References, calls, binary expressions, and other runtime computations are not
valid defaults.

## Named arguments

Named arguments use `name: expression` and may reorder fixed parameters:

```thp
echo retry(delays: [2, 4], attempts: 2) . "";
```

Argument expressions are evaluated once in source order, independent of the
parameter order. The compiler diagnoses unknown names, duplicate bindings,
missing required arguments, and positional arguments following a named
argument. Named binding applies to statically resolved functions, instance
methods, constructors, static methods, and native built-ins.

Parameter names are therefore part of the current source-level call contract.

## Variadic parameters

A final parameter written `T ...$values` collects zero or more extra
positional arguments into a fresh `vector<T>`:

```thp
function total(int $base = 0, int ...$values): int {
    foreach ($values as $value) {
        $base = $base + $value;
    }
    return $base;
}

echo total(1, 2, 3) . "";
```

A callable may have at most one variadic parameter. It must be final and
cannot have a default. Named arguments cannot target it. Call-site unpacking
such as `total(...$values)` is rejected during parsing; by-reference
parameters are not implemented.

## Return values

A non-`void` function returns a value compatible with its declared return type.
A `void` function returns no useful value. `never` describes a function that
does not complete normally.

## Anonymous callables

`callable<P1, ..., R>` is a value type whose final type argument is its return
type; `callable<R>` accepts no arguments. Parameters are invariant and must
match exactly. A bare `callable` has no executable type contract. Callable
values may be passed as parameters, returned, stored, and invoked with
positional arguments. Invocation checks argument count and types at the call
site. Named arguments, defaults, and variadics are not supported when invoking
through a callable value. A named function with only required, fixed
parameters can be used as a callable value.

```thp
function apply(callable<int, int> $operation, int $value): int {
    return $operation($value);
}

$offset: int = 3;
$add = fn(int $value): int => $value + $offset;
echo apply($add, 4);
```

Arrow functions capture referenced outer locals implicitly. Block closures
list captures explicitly with `use ($name, ...)`:

```thp
$add = function (int $value) use ($offset): int {
    return $value + $offset;
};
```

Both forms require parameter and return types. A capture copies the value when
the closure is created. Later reassignment of the outer local does not update
the capture. Collection captures keep copy-on-write storage; object captures
keep the same object identity. Captures are owned by the closure and released
when it is reclaimed, including through cycle collection. Duplicate or
unknown explicit captures are errors at the closure expression. Unlisted outer
locals in a block closure are unresolved inside it. Arrow captures include
values needed by nested arrow closures.

An allocation failure while creating a closure stops evaluation with a runtime
allocation error. A thrown exception from a callable propagates through its
invocation with normal stack traces and `try`/`finally` cleanup.

## See also

- [Types](thp:guide.languageTypes)
- [Expressions](thp:guide.languageExpressions)
- [Classes and objects](thp:guide.languageClassesAndObjects)
