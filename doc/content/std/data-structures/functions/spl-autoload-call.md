---
kind: function
id: std.spl.spl_autoload_call
title: spl_autoload_call
summary: Runs the registered autoload callbacks for a type name.
name: spl_autoload_call
order: 9
typeParameters: []
parameters:
  - name: class
    type: string
    description: The qualified class, interface, or enum name to request.
returns:
  type: void
  description: This function does not return a value.
errors:
  - description:
      An error thrown by a loader is propagated immediately, and later callbacks
      are not called. Invalid type-name syntax and recursive attempts to load the same
      type may also fail; concrete error types are not yet finalized.
related: []
status: experimental
availability: proposed
notice:
  This PHP-inspired contract is proposed and is not yet implemented in this
  repository. Missing-type and re-entrancy behavior may change.
version: "0.1"
module: data-structures
---

`spl_autoload_call()` manually runs the autoload queue for a qualified type
name.

## Behavior

Loaders run in queue order. After each callback returns, the runtime checks
whether `$class` has been declared and stops the queue when it has.

The function always starts an explicit autoload attempt, even if code is not
currently constructing or inspecting the named type. A leading namespace
separator is normalized away before callbacks receive the name.

If no callback declares `$class`, the function returns normally. Code that
requires the declaration may subsequently report that the type does not exist.

## Example

```thp
spl_autoload_register(function (string $class): void {
    require "./plugins/" . str_replace("\\", "/", $class) . ".thp";
});

spl_autoload_call("Plugins\\Metrics\\Exporter");
```

After the call, `Plugins\Metrics\Exporter` is available if a registered loader
declared it.

## See also

- [`spl_autoload_register()`](thp:std.spl.spl_autoload_register)
- [`spl_autoload_functions()`](thp:std.spl.spl_autoload_functions)
- [`spl_autoload()`](thp:std.spl.spl_autoload)
- [PHP `spl_autoload_call()`](https://www.php.net/manual/en/function.spl-autoload-call.php)
