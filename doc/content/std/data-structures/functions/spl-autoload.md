---
kind: function
id: std.spl.spl_autoload
title: spl_autoload
summary: Loads a type by searching include paths with configured filename suffixes.
name: spl_autoload
order: 8
typeParameters: []
parameters:
  - name: class
    type: string
    description: The qualified type name to load.
  - name: file_extensions
    type: ?string
    description: A comma-separated suffix list used only for this call.
    default: "null"
returns:
  type: void
  description: This function does not return a value.
errors:
  - description:
      The call may fail when a matching file cannot be loaded, a loaded file is
      invalid, or no searched file declares $class. Concrete error types and the
      distinction between a missing file and a missing declaration are not yet
      finalized.
related: []
status: experimental
availability: proposed
notice:
  This PHP-inspired contract is proposed and is not yet implemented in this
  repository. Path mapping, include-path integration, and errors may change.
version: "0.1"
module: data-structures
---

`spl_autoload()` is the default loader registered when
[`spl_autoload_register()`](thp:std.spl.spl_autoload_register) receives no callback.

## Behavior

For each configured include path, the loader derives a relative filename from
`$class` and tries the suffixes in their listed order. It stops after a file
declares the requested class, interface, or enum.

The exact namespace-to-path mapping and default suffix list are not yet
finalized. Applications that require predictable PSR-style mapping should
register an explicit callback instead of relying on this default.

## Example

Use `.thp` files for one explicit lookup:

```thp
spl_autoload("App\\Domain\\Invoice", ".thp");
```

The supplied suffix does not replace the process-wide suffix configuration.

## See also

- [`spl_autoload_register()`](thp:std.spl.spl_autoload_register)
- [`spl_autoload_call()`](thp:std.spl.spl_autoload_call)
- [`spl_autoload_extensions()`](thp:std.spl.spl_autoload_extensions)
- [PHP `spl_autoload()`](https://www.php.net/manual/en/function.spl-autoload.php)
