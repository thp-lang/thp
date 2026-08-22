---
kind: guide
id: guide.gettingStarted
title: Getting started
summary: Build the experimental THP toolchain and run a small typed program.
nav:
  section: learn
  order: 10
status: experimental
availability: implemented
---

## Your first program

Download the archive for your platform from the project's GitHub Releases page,
extract it, and add the archive's `bin` directory to `PATH`. Confirm the
installation before continuing:

```sh
thp --version
thp --help
```

Create `hello.thp`:

```thp
<?thp

function greet(string $name): string {
    return "Hello, " . $name . "!\n";
}

echo greet("world");
```

Type-check and run the program:

```sh
thp check hello.thp
thp run hello.thp
```

When working from a source checkout with Rust 1.88 or newer, the equivalent
development commands are:

```sh
cargo run -p thp-cli -- run hello.thp
```

Type-check without running, or inspect a measured compiler stage:

```sh
cargo run -p thp-cli -- check hello.thp
cargo run -p thp-cli -- inspect --emit=mir --metrics=human hello.thp
```

Enable the verified persistent bytecode cache for repeated runs:

```sh
cargo run -p thp-cli -- run --opcache=.thp-cache --metrics=human hello.thp
cargo run -p thp-cli -- cache-prune --opcache=.thp-cache --max-bytes=268435456
```

`--engine=auto` is the default. It selects the baseline Cranelift tier for its
semantics-preserving scalar subset and otherwise uses the reference VM.
`--engine=jit` makes unsupported native-code input an explicit error.

THP does not emit PHP or execute through PHP's engine. The current interpreter
supports only the vertical core listed under
[implementation status](thp:guide.implementationStatus).

## Next step

Run the tested `examples/project` project to see namespaces, typed interfaces
and classes, collections, project autoloading, and structured errors together:

```sh
thp run --project=examples/project main.thp
```

Then define limits and target-specific settings in
[Project configuration](thp:guide.projectConfiguration), read the
[language overview](thp:guide.languageOverview), and use availability badges to
avoid mistaking design proposals for executable APIs.
