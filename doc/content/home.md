---
kind: home
id: docs.home
title: THP Documentation
summary: THP is an experimental, statically typed, PHP-shaped language for greenfield command-line programs.
status: experimental
availability: implemented
---

## When to choose THP over PHP

THP keeps familiar syntax while making collection shapes and conversions more
explicit:

| Language behavior                                      | THP    | PHP    |
| ------------------------------------------------------ | ------ | ------ |
| Separate native `vector<T>` and `map<K, V>` types      | ✅ Yes | ❌ No  |
| One general-purpose `array` type                       | ❌ No  | ✅ Yes |
| Compile-time collection key and value types            | ✅ Yes | ❌ No  |
| Numeric-string key `"8"` always remains a string       | ✅ Yes | ❌ No  |
| Magic array-key conversion (`"8"` → `8`, `true` → `1`) | ❌ No  | ✅ Yes |
| Heterogeneous values without an explicit union         | ❌ No  | ✅ Yes |
| Boolean-only conditions                                | ✅ Yes | ❌ No  |
| Loose `==` type juggling                               | ❌ No  | ✅ Yes |
| Copy-on-write collection values                        | ✅ Yes | ✅ Yes |
| Boolean output as the literals `true` and `false`      | ✅ Yes | ❌ No  |

For example, a THP `map<string, int>` keeps `"8"` as a string key and rejects
an integer key during type checking. PHP arrays accept both forms and apply
their defined key conversions. See [Types](thp:guide.languageTypes) and
[Operators](thp:guide.languageOperators) for the complete THP contracts.

## What runs today

- **Typed language core.** Inferred and annotated variables, functions,
  control flow, unions, nullable types, vectors, and insertion-ordered maps.
- **Objects and failures.** Classes, interfaces, traits, inheritance, virtual
  dispatch, structured exceptions, `finally`, `match`, and deterministic
  `using` cleanup.
- **Static projects.** Namespaces, imports, configured autoload discovery,
  cross-file declarations, dependency graphs, and reusable prepared projects.
- **Managed runtime and streams.** Checked integers, reference-counted values,
  cycle collection, binary-safe memory and temporary streams, and configurable
  request limits.
- **Verified execution.** A bytecode interpreter, content-addressed OPcache,
  frozen project execution, and a baseline Cranelift JIT for its safe scalar
  subset.
- **Tools and embedding.** Structured diagnostics, inspectable compiler stages,
  human or JSON metrics, a safe Rust embedding API, and a versioned C ABI.

The [implementation status](thp:guide.implementationStatus) is the detailed
authority for accepted syntax and executable behavior.

## Try it locally

Download an archive from
[GitHub Releases](https://github.com/thp-lang/thp/releases), add its `bin`
directory to `PATH`, and run the example above:

```sh
thp --version
thp check hello.thp
thp run hello.thp
```

The [getting-started guide](thp:guide.gettingStarted) also covers building from
source, inspecting compiler stages, selecting the VM or JIT, and enabling the
persistent cache.

## Clear experimental boundaries

> THP 0.1 is for command-line experiments. It is not production-ready, is not
> a PHP-compatible replacement, does not execute through the PHP engine, and is
> not yet a web backend.

The baseline JIT deliberately supports only a scalar subset and falls back to
the VM in automatic mode. Much of the broader standard library remains a
proposal. Availability badges distinguish implemented, partial, and proposed
contracts throughout these docs.

## Choose your path

- **[Start using THP](thp:guide.gettingStarted).** Install the toolchain and run
  a small typed program.
- **[Start a project](thp:guide.startProject).** Create `thp.toml`, map a
  namespace, and compile a multi-file command-line application.
- **[Coming from PHP](thp:guide.phpDevelopers).** Learn which syntax carries
  over and where THP deliberately uses different types, loading, and runtime
  behavior.
- **[Read the language reference](thp:guide.languageOverview).** Learn the
  syntax and follow availability badges for the current feature boundary.
- **[Check implementation status](thp:guide.implementationStatus).** See the
  precise executable surface and deliberately pending work.
- **[Explore the internals](thp:guide.internalsOverview).** Follow source
  through the compiler, bytecode verifier, VM, OPcache, and JIT.
