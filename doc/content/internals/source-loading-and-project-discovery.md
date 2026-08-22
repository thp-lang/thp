---
kind: guide
id: guide.sourceLoading
title: Source loading and project discovery
summary: See how THP selects source files, assigns module identities, and builds the compilation input.
nav:
  section: internals
  order: 20
status: experimental
availability: implemented
notice: >-
  Source providers and project discovery are implemented for the current static
  module model.
---

Compilation begins by producing a `SourceFile` for every input. A single-file
command reads exactly the requested path and requires valid UTF-8 source. A
project command first loads `thp.toml`, resolves its ordered `[autoload]`
mappings, and asks a source provider to enumerate the entry file and matching
modules.

```toml
[autoload]
"Example\\" = ["src"]
```

```thp
<?thp

use Example\Greeting;

echo Greeting::message();
```

Project paths are resolved from the selected project root. The filesystem
provider canonicalizes discovered paths, rejects files outside configured
roots, assigns deterministic module IDs, and marks exactly one module as the
entry. Hosts embedding THP can supply the same module bytes through the
`ModuleSourceProvider` interface instead of the filesystem.

## Output of this stage

The compiler retains a source map and one project unit per module. Each unit
contains its module identity, source ID, bytes decoded as text, later tokens and
AST, content hash, and cache-reuse report. Diagnostics refer to source IDs and
byte spans, so presentation layers can render the correct path and line even
after units are linked.

Discovery is measured separately from loading. Failure to read a file, decode
UTF-8, load project configuration, or enumerate the provider is a loading
failure. It prevents lexing. Language problems inside a successfully loaded
file become structured diagnostics in later phases.

## Project determinism

Discovery order does not control semantic order. After parsing, the module
graph establishes deterministic dependency order and legal declaration cycles.
The entry module may contain executable top-level statements; imported modules
currently contain declarations only.

The configured root also supplies runtime-relative filesystem context. The
runtime does not change the process working directory to implement project
paths.

## Design choices compared with PHP

PHP treats loading as an executable operation: `include` and `require` may be
conditional, their paths may be computed, and an autoloader is ordinary user
code. This makes applications and plugin systems highly composable at runtime,
but the complete program is not necessarily known when the entry script starts
compiling.

THP instead makes the project root, entry file, and namespace-to-directory
mappings compiler inputs. Discovery finishes before type analysis, and every
dependency receives a stable module ID and content hash. That enables complete
cross-file diagnostics, deterministic graph order, granular cache keys, and a
frozen deployment that need not scan source directories. The tradeoff is that
conditional includes and user-defined loading logic are not part of the
current module model.

Next, every loaded source file enters [lexing](thp:guide.lexingInternals).
