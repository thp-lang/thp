# Resource and stream proposal fixtures

These PHPT files describe and test the observable behavior of THP's initial
resource and stream model. The `thp-test` runner recursively discovers,
compiles, and executes every fixture through the THP VM during
`cargo test --workspace`.

Run this directory directly from the repository root:

```sh
cargo run -p thp-test -- tests/phpt/resources
cargo run -p thp-test -- --max-instructions=100000 tests/phpt/resources
```

## Supported sections

Every fixture requires `TEST`, exactly one program section, and exactly one
expectation section:

- `--TEST--` names one behavior.
- `--FILE--` contains a complete `<?thp` program.
- `--FILE_EXTERNAL--` names a UTF-8 THP file inside the fixture directory.
- `--EXPECT--` compares bytes exactly after portable trimming and CRLF
  normalization.
- `--EXPECTF--` supports `%e`, `%s`, `%S`, `%a`, `%A`, `%w`, `%i`, `%d`, `%x`,
  `%f`, `%c`, `%0`, and `%r...%r`.
- `--EXPECTREGEX--` uses full-output, dot-all Rust byte-regex syntax.
- `--SKIPIF--` runs before the test. Empty output continues; `skip` followed by
  an optional reason skips; any other output or execution failure BORKs.
- `--CLEAN--` runs after every attempted main program. It must succeed without
  output.
- `--CONFIG--` contains inline THP project TOML.
- `--STDIN--` supplies the binary request body exposed through `thp:/input`.
- `--CREDITS--` and `--DESCRIPTION--` are accepted as metadata.

Compile and runtime diagnostics are appended to output already emitted by the
program before matching. A negative test therefore passes when that combined
byte stream matches its expectation.

`CONFIG` accepts the common core tables currently understood by `thp.toml`:

```toml
[memory]
limit = "128M"

[request]
post_max_size = "8M"
max_stack_depth = 512
max_open_handles = 256

[time]
max_input = "60s"
max_execution = "2s"
```

For a contained `--FILE_EXTERNAL--` project fixture, `CONFIG` may also map
static source modules relative to the fixture directory:

```toml
[autoload]
"App\\" = "src/"
```

Mapped modules are compiled and linked with the external entry. Inline
`--FILE--` fixtures cannot use autoload mappings.

The runner enforces the configured execution, managed-heap, request-input,
stack-depth, and open-handle limits. `SKIPIF` and `CLEAN` receive an empty
request body; `FILE` receives `STDIN` when present. Target tables BORK because
the runner has no target selection. Extension tables skip because extensions
are not loaded by the VM runner. Tests containing `INI`, `EXTENSIONS`, `ARGS`,
`ENV`, or recognized web-SAPI/debugger sections skip as unsupported host
capabilities. Unknown or duplicate sections BORK.

## THP source

PHPT is only a container: source inside `FILE`, `SKIPIF`, and `CLEAN` is normal
THP and is never rewritten. In particular, generic syntax needs no PHPT
parameters:

```phpt
--TEST--
Generic type syntax is ordinary THP inside PHPT
--FILE--
<?thp

$values: vector<int> = [41, 42];
echo $values[1] . "\n";
--EXPECT--
42
```

The fixtures must not be passed to PHP's interpreter because `<?thp`, typed
signatures, capability interfaces, and `using` are THP syntax.
