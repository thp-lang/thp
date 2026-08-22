---
kind: guide
id: guide.gettingStarted
title: Getting started
summary: Install THP, run a first typed program, and understand the development loop.
nav:
  section: learn
  order: 10
status: experimental
availability: implemented
---

This guide takes you from an installed `thp` command to a checked and running
program. THP 0.1 is intended for command-line experiments. It does not run
through PHP, install into a PHP server, or provide a production web stack.

## Install a release archive

Download the archive for your platform from the project's
[GitHub Releases](https://github.com/thp-lang/thp/releases) page. Extract it and
add the archive's `bin` directory to `PATH`.

Open a new terminal and confirm which binary will run:

```sh
thp --version
thp --help
```

If the shell cannot find `thp`, use the full path to the extracted binary or
correct the `PATH` entry before continuing. THP does not require a PHP
installation because the archive contains a standalone compiler and runtime.

## Build from source

To work from a source checkout, install Rust 1.88 or newer and build the CLI
crate from the repository root:

```sh
cargo build --release -p thp-cli
target/release/thp --version
```

During compiler development, `cargo run` can build and invoke the same CLI in
one command:

```sh
cargo run -p thp-cli -- --version
```

The examples below use `thp`. Replace that word with `target/release/thp`, or
with `cargo run -p thp-cli --`, when running a local source build.

## Write your first program

Create a directory for the experiment and save this as `hello.thp`:

```thp
<?thp

function greet(string $name): string {
    return "Hello, " . $name . "!\n";
}

echo greet("world");
```

Every THP source file starts with `<?thp`. Variables retain the familiar `$`
prefix, while parameters and return values participate in static type
checking. A `.thp` file is not a `.php` file with additional annotations.

Check the program without executing it:

```sh
thp check hello.thp
```

No output means that checking succeeded. Now run it:

```sh
thp run hello.thp
```

The result is:

```text
Hello, world!
```

`run` performs the same front-end checks before execution, so the usual edit
loop is to run `thp check` in fast feedback or automation and `thp run` when
you want observable program output.

## See static checking in action

Change the call to pass an integer:

```thp
echo greet(42);
```

`thp check hello.thp` rejects the argument before the program starts. Restore
the string after trying the diagnostic. THP deliberately does not use PHP's
weak scalar coercions to turn this call into `greet("42")`.

Conditions are checked in the same way. Write a boolean expression rather than
depending on PHP truthiness:

```thp
$names: vector<string> = ["Ada", "Grace"];

if (count($names) > 0) {
    echo "The list is not empty.\n";
}
```

The declared `vector<string>` also prevents a later integer from being stored
in the collection. See [THP for PHP developers](thp:guide.phpDevelopers) for a
larger migration-oriented comparison.

## Inspect what the compiler produced

`thp inspect` stops after a selected compiler representation and prints it:

```sh
thp inspect --emit=tokens hello.thp
thp inspect --emit=ast hello.thp
thp inspect --emit=hir hello.thp
thp inspect --emit=mir hello.thp
thp inspect --emit=bytecode hello.thp
```

Add `--metrics=human` to `check`, `inspect`, or `run` to report compiler and
runtime stage measurements. JSON is available for tools:

```sh
thp check --metrics=human hello.thp
thp run --metrics=json hello.thp
```

These commands are useful for understanding the implementation; they are not
required for ordinary programs.

## Choose an execution engine

`--engine=auto` is the default. It selects the baseline Cranelift JIT only when
the whole program fits its semantics-preserving scalar subset and otherwise
uses the reference bytecode VM.

```sh
thp run --engine=vm hello.thp
thp run --engine=auto hello.thp
```

`--engine=jit` turns unsupported native-code input into an explicit error. It
does not make unsupported language features available. Use the VM while
learning unless you specifically want to exercise the native tier.

For an untrusted or accidentally non-terminating experiment, the VM can stop
after an instruction budget:

```sh
thp run --engine=vm --max-instructions=100000 hello.thp
```

## Single files and projects

A single file is enough for a small experiment. When the selected project root
does not contain `thp.toml`, THP reads only the path named on the command line.
It does not search parent directories for configuration.

A multi-file project adds an explicit `thp.toml`, an entry file, namespaced
source files, and static namespace-to-directory mappings. THP then discovers
and checks the complete module set before running the entry file.

Continue with [Start a project](thp:guide.startProject) to build that layout
from an empty directory. The tested `examples/project` checkout is also ready
to run:

```sh
thp run --project=examples/project main.thp
```

## Cache repeated runs

The persistent OPcache stores verified bytecode by content and compiler
identity. It is optional for development:

```sh
thp run --opcache=.thp-cache --metrics=human hello.thp
thp cache-prune --opcache=.thp-cache --max-bytes=268435456
```

Project deployments can warm a complete linked artifact and then run it
without scanning mapped source directories:

```sh
thp cache-warm --project=examples/project --opcache=.thp-cache main.thp
thp run --frozen --project=examples/project --opcache=.thp-cache main.thp
```

Caching does not relax source validation or bytecode verification. It is an
execution optimization, not a package manager or PHP OPcache compatibility
mode.

## Where to go next

- [Start a project](thp:guide.startProject) builds a multi-file application
  with namespaces and autoload discovery.
- [THP for PHP developers](thp:guide.phpDevelopers) maps familiar PHP habits to
  THP's stricter contracts.
- [Project configuration](thp:guide.projectConfiguration) documents every
  current `thp.toml` field.
- [Implementation status](thp:guide.implementationStatus) is the detailed
  authority for syntax that executes today.
