---
kind: guide
id: guide.projectConfiguration
title: Project configuration
summary: Define package discovery, runtime limits, target overrides, and extension settings in project TOML.
nav:
  section: learn
  order: 40
status: experimental
availability: implemented
notice: >-
  The configuration loader and lock format are implemented, but THP does not yet enforce these limits at runtime or validate extension-owned settings.
---

THP project settings live beside the project source. The project root must
contain `thp.toml`. It may also contain `thp.local.toml` for settings that
belong only to one checkout.

The loader does not search for global or user configuration and does not read
configuration overrides from the environment. The caller chooses the project
root explicitly.

If you are creating your first multi-file application, begin with
[Start a project](thp:guide.startProject). Most projects initially need only an
autoload mapping:

```toml
[autoload]
"App\\" = "src/"
```

All runtime-limit tables are optional. An empty `thp.toml` is valid for a
project that uses only its entry file, although a single-file command needs no
project file at all.

## Autoload mappings

The `[autoload]` table maps a case-sensitive namespace prefix to one directory
or an ordered list of directories:

```toml
[autoload]
packages = "vendor/"
"App\\" = "src/"
```

A non-empty prefix ends in `\` and contains valid THP name segments. The
backslash is escaped inside a TOML basic string. Relative directories resolve
from the selected project root.

For a mapping from `App\` to `src/`, the path
`src/Service/Client.thp` has module ID `App\Service\Client` and declares
`namespace App\Service;`. The compiler discovers `.thp` files in mapped
directories before type checking. This is not runtime autoloading: the
mappings name source roots but do not invoke user callbacks.

`thp.local.toml` may add mappings or replace a project mapping with the same
prefix for that checkout. Discovery rejects ambiguous logical or physical
module mappings.

### Installed packages

The reserved `packages` key accepts one project-relative directory or an
ordered list:

```toml
[autoload]
packages = ["vendor/", "generated-vendor/"]
"App\\" = "src/"
```

THP scans exactly `<package-root>/<vendor>/<package>/thp.toml`, with packages
ordered by `vendor/package` inside each configured root. Files at the vendor
and package-name levels are ignored. Every package directory must contain a
valid `thp.toml`.

Package roots from `thp.local.toml` are appended to those from `thp.toml`.
Exact duplicate paths keep their first position.

A package contributes only its explicit namespace mappings. Their directories
resolve relative to that package, so this package manifest:

```toml
[autoload]
"Acme\\Clock\\" = ["src/", "generated/"]
```

installed at `vendor/acme/clock` contributes
`vendor/acme/clock/src/` and `vendor/acme/clock/generated/`. THP does not read
the package's `thp.local.toml` or follow its `autoload.packages` setting.
Nested package trees therefore are not discovered.

An exact namespace-prefix collision between the project and a package, or
between two packages, is an error reported at the later package manifest's
mapping. One manifest may still map its prefix to multiple ordered
directories. Package roots cannot be absolute or escape the project through
`..`; symlinked package directories remain usable.

THP discovers already-installed source only. It does not download packages,
select versions, access a registry, or replace a package manager.

## Project schema

Settings use domain-specific tables:

```toml
[memory]
limit = "200M"

[request]
post_max_size = "8M"
max_stack_depth = 512
max_open_handles = 256

[time]
max_input = "60s"
max_execution = "30s"

[extensions.example]
custom_key = "value"

[targets.cli.memory]
limit = "400M"

[targets.cli.extensions.example]
custom_key = "cli-value"
```

All core tables and fields are optional, but `thp.toml` itself is required.
Built-in defaults complete an omitted common profile:

| Setting                    | Default |
| -------------------------- | ------- |
| `memory.limit`             | `128M`  |
| `request.post_max_size`    | `8M`    |
| `request.max_stack_depth`  | `512`   |
| `request.max_open_handles` | `256`   |
| `time.max_input`           | `60s`   |
| `time.max_execution`       | `30s`   |

Unknown core tables and fields are errors. THP does not silently interpret
unknown settings as PHP configuration.

## Sizes, durations, and unlimited values

Sizes are non-negative whole numbers with an optional case-insensitive binary
unit: `K`, `M`, or `G`. A value with no suffix is a number of bytes. For
example, `2M` is 2 × 1,048,576 bytes.

Durations are non-negative whole numbers followed by `s`, `m`, or `h` for
seconds, minutes, or hours. Units are case-sensitive.

The literal `unlimited` removes a limit. Any zero value written with an
accepted unit, such as `0M` or `0s`, is also canonicalized to unlimited. Bare
`0` is accepted for a size because the size suffix is optional; durations
still require a unit. Fractions, negative values, whitespace, composite
durations such as `1h30m`, unsupported suffixes, and values that overflow an
unsigned 64-bit integer are rejected.

Stack depth and open handles use bare unsigned integers. Zero makes either
count unlimited. Stack depth counts logical THP call frames rather than native
Rust or operating-system frames. Handle aliases share one stream cell and
therefore consume one open-handle slot.

`memory.limit` covers request-owned THP cells and payload capacity, including
strings, collections, object properties, exceptions, memory streams, and cycle
collector metadata. `request.post_max_size` and `time.max_input` apply to the
SAPI body or PHPT `--STDIN--` consumed through `thp:/input`; source modules
and ordinary file reads are not request input. Program output is streamed to
the host and has no total-size configuration setting.

## Targets

A table below `targets` overrides the common profile for one deployment
target:

```toml
[targets.web.time]
max_execution = "10s"

[targets.worker.time]
max_execution = "1h"
```

Target names begin with a lowercase ASCII letter and contain only lowercase
ASCII letters, digits, `_`, or `-`. The name `default` is reserved. Targets do
not inherit from one another. Selecting a target that has no declaration uses
the common profile.

## Local overrides and precedence

`thp.local.toml` has the same schema as `thp.toml` and is optional. Resolve a
selected target in this order:

1. Built-in defaults.
2. Common values from `thp.toml`, then matching common values from
   `thp.local.toml`.
3. Values for the selected target from `thp.toml`, then matching target values
   from `thp.local.toml`.

A later core value replaces the earlier value. Extension tables merge
recursively; a later scalar or array replaces the earlier value at the same
key.

There is no target inheritance in this version.

## Extension settings

Each key below `extensions` names an extension and must contain a TOML table:

```toml
[extensions.database]
hosts = ["db-1", "db-2"]

[extensions.database.pool]
minimum = 2
maximum = 20
```

Extension names follow the same identifier rules as targets, including the
reserved `default` name. Nested extension data may contain arbitrary TOML
values. The core loader preserves and merges this data but does not apply an
extension-owned schema. An extension can decode and validate its table only
when it requests the configuration.

## Generated lock file

During deployment preparation, run `thp lock [--project=DIR]` to generate
`thp.lock`. The fingerprint covers the required project file, the exact
presence and contents of the optional local file, discovered package
membership, and every package `thp.toml`. The versioned text lock stores
configured package roots, every resolved namespace mapping, and fully resolved
common and target profiles. Core sizes are canonical byte counts, core
durations are canonical seconds, and mappings, targets, and extensions have
deterministic lexical ordering.

Extension data is stored as length-delimited canonical TOML. Startup code can
load core records and skip or retain these payloads without parsing them.

Lock generation uses atomic replacement and does not rewrite identical output.
On Unix, a generated lock is readable and writable only by its owner.

Without `thp.lock`, development commands scan installed manifests and keep the
resolved mappings in memory. When the file exists, commands that compile or
load a project require it to be valid and fresh, then use its mappings without
parsing package manifests. A stale, corrupt, or unsupported lock is an error;
these commands do not regenerate it automatically. `thp lock` itself always
reads the live configuration and package manifests to create or replace the
lock. Regenerate after either root configuration file changes or after a
package is added, removed, or changes its manifest. Older locks using the
previous version-1 layout must also be regenerated; the lock version remains 1
while this format is experimental.

Add both generated and checkout-local files to the project root `.gitignore`:

```gitignore
/thp.local.toml
/thp.lock
```

Project templates must include the same rules. The configuration library never
edits a project's `.gitignore`.

Runtime enforcement of core limits and extension-specific schema validation
remain unimplemented.
