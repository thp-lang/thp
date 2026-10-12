---
kind: home
id: docs.home
title: THP Documentation
summary: Write familiar PHP-shaped code with generic collections, checked types, and a standalone runtime. THP is experimental and built for command-line programs today.
status: experimental
availability: implemented
---

## Generics you can use today

Declare the types your collections hold. THP checks keys and values before the
program runs, while keeping familiar indexing and `foreach` syntax:

```thp
<?thp
$scores: map<string, int> = {"Ada" => 10, "Grace" => 9};
$names: vector<string> = ["Ada", "Grace"];

foreach ($names as $name) {
    echo $name . ": " . $scores[$name] . "\n";
}
```

Generic classes carry the same type through properties, constructors, and
methods:

```thp
<?thp
class Box<T> {
    public T $value;

    public function __construct(T $value) {
        $this->value = $value;
    }

    public function value(): T {
        return $this->value;
    }
}

$answer = new Box<int>(42);
echo $answer->value();
```

Explore [collection types](thp:guide.languageTypes) and
[generic classes](thp:guide.languageClassesAndObjects) in the language guide.
Generic functions and methods are still proposals; see the
[implementation status](thp:guide.implementationStatus) for the precise
executable surface.

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

On Linux x86-64 with glibc, download the prebuilt THP 0.9.0 release and its
checksum manifest:

```sh
curl --proto '=https' --proto-redir '=https' --tlsv1.2 -fsSL \
  -O https://github.com/thp-lang/thp/releases/download/v0.9.0/thp-0.9.0-linux-x86_64.tar.gz \
  -O https://github.com/thp-lang/thp/releases/download/v0.9.0/SHA256SUMS
```

Verify the exact archive name:

```sh
awk '$2 == "thp-0.9.0-linux-x86_64.tar.gz" { print }' SHA256SUMS |
  sha256sum -c -
```

Continue only if it prints `thp-0.9.0-linux-x86_64.tar.gz: OK`. Extract the
archive and install the binary in `~/.local/bin`:

```sh
tar -xzf thp-0.9.0-linux-x86_64.tar.gz
install -Dm755 thp-0.9.0-linux-x86_64/bin/thp "$HOME/.local/bin/thp"
```

Add `~/.local/bin` to `PATH` for this terminal. Add the same line to `~/.bashrc`
for Bash or `~/.zshrc` for Zsh to keep it available in new terminals:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

Save the collection example as `collections.thp`, then run:

```sh
thp --version
thp check collections.thp
thp run collections.thp
```

The [getting-started guide](thp:guide.gettingStarted) covers macOS and Windows
archives, manual installation, building from source,
compiler stages, the VM or JIT, and the persistent cache.

## Clear experimental boundaries

> THP is for command-line experiments. It is not production-ready, is not
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
