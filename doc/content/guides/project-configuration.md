---
kind: guide
id: guide.projectConfiguration
title: Project configuration
summary: Define runtime limits, target overrides, and extension settings in project TOML.
nav:
  section: learn
  order: 20
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

Configuration tooling generates `thp.lock` from the required project file and
the exact presence and contents of the optional local file. The versioned text
lock stores fully resolved common and target profiles. Core sizes are
canonical byte counts, core durations are canonical seconds, and targets and
extensions have deterministic lexical ordering.

Extension data is stored as length-delimited canonical TOML. Startup code can
load core records and skip or retain these payloads without parsing them.

Lock generation uses atomic replacement and does not rewrite identical output.
On Unix, a generated lock is readable and writable only by its owner. This is
important because merged local extension settings can contain credentials or
other secrets.

Loading a lock checks its source fingerprint. A missing, stale, corrupt, or
unsupported lock is an error; the loader does not regenerate it automatically.
Regenerate the lock explicitly after `thp.toml` changes, or after
`thp.local.toml` appears, disappears, or changes.

Add both generated and checkout-local files to the project root `.gitignore`:

```gitignore
/thp.local.toml
/thp.lock
```

Project templates must include the same rules. The configuration library never
edits a project's `.gitignore`.

Runtime enforcement of core limits and extension-specific schema validation
remain unimplemented.
