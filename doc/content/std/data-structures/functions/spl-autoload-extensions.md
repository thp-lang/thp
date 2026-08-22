---
kind: function
id: std.spl.spl_autoload_extensions
title: spl_autoload_extensions
summary: Reads or replaces the filename suffixes used by the default autoloader.
name: spl_autoload_extensions
order: 10
typeParameters: []
parameters:
  - name: file_extensions
    type: ?string
    description: A comma-separated list of filename suffixes.
    default: "null"
returns:
  type: string
  description:
    The active suffix list as a comma-separated string. When a non-null value
    is supplied, the returned string is the newly configured list.
errors:
  - description:
      An invalid or empty suffix entry may cause the call to fail. Exact
      validation rules and concrete error types are not yet finalized.
related: []
status: experimental
availability: proposed
notice:
  This PHP-inspired contract is proposed and is not yet implemented in this
  repository. The default suffix list and validation rules may change.
version: "0.1"
module: data-structures
---

`spl_autoload_extensions()` inspects or changes the suffix list used by
[`spl_autoload()`](thp:std.spl.spl_autoload).

## Behavior

The setting is process-wide and affects later calls to
[`spl_autoload()`](thp:std.spl.spl_autoload) that do not supply their own suffix list.
Suffixes are tried from left to right.

Each suffix should include its leading dot. Whitespace is significant and
should not appear around commas. The default list is not yet finalized.

Changing the setting does not invoke a loader, retry earlier failures, or
change explicitly registered callback behavior.

## Example

```thp
spl_autoload_extensions(".thp,.generated.thp");
spl_autoload_register();
```

The default loader searches for `.thp` files before `.generated.thp` files.

## See also

- [`spl_autoload()`](thp:std.spl.spl_autoload)
- [`spl_autoload_register()`](thp:std.spl.spl_autoload_register)
- [PHP `spl_autoload_extensions()`](https://www.php.net/manual/en/function.spl-autoload-extensions.php)
