---
kind: guide
id: guide.phpDevelopers
title: THP for PHP developers
summary: Learn which PHP instincts carry over, which contracts are stricter, and which ecosystem features are absent.
nav:
  section: learn
  order: 20
status: experimental
availability: implemented
notice: >-
  This comparison describes the current executable THP core. Familiar syntax
  is not a promise of PHP source, runtime, library, or extension compatibility.
---

THP looks deliberately familiar to a PHP developer: source begins with an
opening tag, variables start with `$`, strings concatenate with `.`, and
functions, classes, namespaces, exceptions, and `foreach` use PHP-shaped
syntax. The similarity is a starting point for learning, not a compatibility
layer.

THP is a separate, statically typed language with its own compiler, bytecode,
VM, object model, module discovery, standard-library contracts, and optional
JIT. Existing PHP applications and Composer packages cannot be renamed to
`.thp` and executed.

## The short comparison

| Concern             | PHP habit                                               | Current THP contract                                              |
| ------------------- | ------------------------------------------------------- | ----------------------------------------------------------------- |
| Execution           | PHP engine, SAPI, or framework                          | Standalone THP compiler and runtime                               |
| File suffix and tag | `.php` and `<?php`                                      | `.thp` and required `<?thp`                                       |
| Types               | Coercive behavior is common and mode-dependent          | Assignments, calls, returns, and operators are statically checked |
| Conditions          | Values may be converted by truthiness                   | Conditions require `bool`                                         |
| Equality            | `==` performs type juggling                             | Operands need matching static types; no PHP type juggling         |
| Arrays              | One ordered `array` shape                               | Separate `vector<T>` and `map<K, V>` values                       |
| Integers            | Platform-sized PHP integers                             | Checked signed 64-bit integers                                    |
| Names               | Class and function lookup is generally case-insensitive | Canonical names and imports are case-sensitive                    |
| Loading             | `include`, `require`, Composer, runtime autoloaders     | Static `[autoload]` discovery before type analysis                |
| Top-level code      | Any included file may execute statements                | Only the project entry module may contain top-level statements    |
| References          | `&` aliases and by-reference parameters                 | Reference syntax and by-reference parameters are not implemented  |
| Globals             | Superglobals and process bindings are built in          | No stable superglobals or CLI predefined variables yet            |
| Ecosystem           | Mature extensions, frameworks, and packages             | An experimental language core; most library pages are proposals   |

## What feels familiar

A small, explicitly typed function is visually close to modern PHP:

```thp
<?thp

function total(int $price, int $quantity): int {
    return $price * $quantity;
}

echo total(12, 3);
```

Blocks use braces, statements end in semicolons, comments use `//` or `/* */`,
and instance access uses `->`. THP also implements nominal classes,
interfaces, traits, exceptions, `try`/`catch`/`finally`, `match`, namespaces,
named arguments, and variadic parameters within the boundaries listed in
[Implementation status](thp:guide.implementationStatus).

The important habit is to read every construct as a THP construct. When the
THP reference does not define behavior, do not fill the gap from PHP.

## Types are program contracts

THP checks the type of each executable expression and binding. A variable may
infer its type from its initializer or state it explicitly:

```thp
$retries = 3;
$label: string = "ready";
```

Both variables keep those types. This is rejected instead of silently changing
the binding or coercing its value:

```thp
$retries = "three";
```

Function arguments and results follow the same rule:

```thp
function labelFor(int $id): string {
    return "item-" . $id;
}
```

String concatenation accepts `string`, `int`, `float`, and `bool` through a
narrow, canonical output conversion. That exception does not make an `int`
assignable to a `string` variable and does not establish general weak typing.

## Replace truthiness with boolean questions

PHP commonly uses a value directly as a condition. THP conditions require a
`bool`, so state the intended question:

```thp
$names: vector<string> = ["Ada"];
$message: ?string = null;

if (count($names) > 0) {
    echo "names are available\n";
}

echo $message ?? "no message";
```

This avoids treating unrelated values such as `0`, `"0"`, an empty
collection, and `null` as variations of the same control-flow signal. Flow
narrowing is still incomplete, so consult the language page and implementation
status before relying on a particular narrowing pattern.

## Use vectors and maps instead of PHP arrays

PHP's `array` combines sequence and dictionary behavior. THP separates those
roles and checks their element types.

A vector is an ordered sequence with integer offsets:

```thp
$names: vector<string> = ["Ada", "Grace"];
$first = $names[0];
```

A map is an insertion-ordered key/value collection and uses braces for its
literal:

```thp
$scores: map<string, int> = {"Ada" => 10, "Grace" => 9};
$scores["Linus"] = 8;
```

Empty literals need an expected generic type:

```thp
$names: vector<string> = [];
$scores: map<string, int> = {};
```

Both shapes support direct native `foreach`, indexing, `count()`, and
variable-rooted element assignment. Vector assignment replaces an existing
offset; it does not append by writing beyond the end. Collections have
copy-on-write value behavior, while object aliases observe the same property
mutations.

## Expect strict operators and checked arithmetic

Do not use `==` as a conversion tool. It currently accepts only operands with
matching static types and performs no PHP type juggling. Convert or parse data
at a deliberate boundary before comparing it.

THP integers are signed 64-bit values on every supported platform. Overflow,
division by zero, and other invalid arithmetic are failures rather than
platform-dependent wraparound behavior. Floating-point output is canonical and
locale-independent.

`echo` accepts exactly one scalar expression. Concatenate a message first:

```thp
echo "count=" . count($names) . "\n";
```

It rejects `null`, collections, objects, `mixed`, and nullable unions. Supply a
fallback or narrow the value before output:

```thp
echo $displayName ?? "anonymous";
```

Use `var_dump()` when inspecting a value and its type during an experiment.

## Treat source loading as compilation

PHP can run a computed `include`, register autoload callbacks, and let an
included file perform initialization. THP discovers a project statically from
`thp.toml`:

```toml
[autoload]
"App\\" = "src/"
```

`src/Service/Greeter.thp` under that mapping has module ID
`App\Service\Greeter` and declares `namespace App\Service;`. Imports create
compile-time dependency edges. Unknown imports and duplicate exports are
reported before execution.

Only the selected entry file may contain executable top-level statements.
Put reusable initialization in a named function and call it from the entry.
There is no current equivalent of Composer autoloading, `include`, `require`,
or arbitrary user-defined runtime loading.

Name resolution is case-sensitive. Type imports and function imports use
separate tables:

```thp
use App\Service\Greeter;
use function App\Support\makeGreeter;
```

THP does not use PHP's runtime global-function fallback. Statically known
prelude functions remain available where the THP language defines them.

## Reconsider shared mutation

PHP references, reference returns, and by-reference parameters are not part of
the executable THP core. Prefer returned values, explicit collection
assignment, or a mutable object whose identity communicates shared state.

```thp
function increment(int $value): int {
    return $value + 1;
}

$count = increment($count);
```

THP objects use reference identity, so two variables holding the same object
observe the same property changes. Native vectors and maps instead detach
shared storage before mutation.

## Do not assume PHP runtime globals or libraries

THP currently defines no stable `$_SERVER`, `$_GET`, `$_POST`, `$_ENV`,
`$argc`, or `$argv`. It is not yet a web backend, and environment, request,
session, cookie, upload, and process APIs need explicit THP contracts before
applications can depend on them.

Likewise, familiar standard-library names in this documentation may describe
proposals rather than executable functions. Read the `availability` badge on
each page and use [Implementation status](thp:guide.implementationStatus) as
the detailed feature authority.

THP also does not load PHP extensions, Composer packages, or framework code.
Choose PHP for an existing PHP application or production service; choose THP
today for a greenfield command-line experiment in this language model.

## Errors occur at different boundaries

Syntax, name, and type mistakes are structured compile diagnostics. Failures
that depend on runtime data, such as an invalid collection offset or checked
arithmetic failure, occur in the VM. User `Exception` descendants and runtime
`Error` descendants participate in typed `try`/`catch` where their contracts
say so.

Compiler, bytecode-verification, instruction-limit, and host failures are not
ordinary source exceptions. Do not expect every failure printed by the CLI to
be catchable inside a THP program.

## A practical migration checklist

When rewriting a small PHP algorithm as THP:

1. Start with a standalone `.thp` entry and the required `<?thp` tag.
2. Add parameter and return types, then make inferred local types consistent.
3. Replace every truthy condition with a `bool` expression.
4. Choose `vector<T>` or `map<K, V>` for each PHP array.
5. Remove implicit scalar coercions and cross-type equality.
6. Replace dynamic includes and autoload callbacks with `thp.toml` mappings.
7. Move imported-file side effects into functions called by the entry.
8. Replace references and hidden globals with explicit values or objects.
9. Verify each needed library feature is marked implemented.
10. Run `thp check` before `thp run` and keep the experiment within THP's
    current command-line scope.

Continue with [Start a project](thp:guide.startProject) for a complete
multi-file layout, or open the [language overview](thp:guide.languageOverview)
for reference material.
