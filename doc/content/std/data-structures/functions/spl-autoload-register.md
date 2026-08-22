---
kind: function
id: std.spl.spl_autoload_register
title: spl_autoload_register
summary: Registers a callback that can load missing type declarations.
name: spl_autoload_register
order: 12
typeParameters: []
parameters:
  - name: callback
    type: ?callable
    description: A callable accepting the requested qualified type name as a string.
    default: "null"
  - name: prepend
    type: bool
    description: Whether to add the callback at the front of the queue.
    default: "false"
returns:
  type: bool
  description: Always true after the callback has been registered.
errors:
  - description:
      Registration fails by throwing when $callback cannot be called with one
      string argument or the runtime cannot modify the loader queue. THP omits PHP's
      legacy $throw parameter rather than exposing an ignored option. The concrete THP
      error types are not yet finalized.
  - description: An error thrown by a loader is propagated and stops the current autoload attempt.
related: []
status: experimental
availability: proposed
notice:
  This PHP-inspired target contract is not yet implemented in this checkout.
  Callback validation and concrete error types are not finalized.
version: "0.1"
module: data-structures
---

`spl_autoload_register()` adds a callback to the process-wide autoload queue.

## Behavior

Callbacks normally run in registration order. When `$prepend` is `true`, the
new callback runs before callbacks already in the queue.

The runtime calls each callback with a qualified class, interface, or enum name,
without a leading namespace separator. After each callback returns, it checks
whether the requested declaration now exists. The queue stops at the first
successful loader.

Registering a callable that is already present is idempotent: the queue keeps its existing entry and does not invoke it twice.

## Example

Place a project loader before loaders registered by dependencies:

```thp
spl_autoload_register(
    function (string $class): void {
        $path = "./src/" . str_replace("\\", "/", $class) . ".thp";
        require $path;
    },
    prepend: true,
);
```

Registration does not call the loader. The first reference to a missing type
starts the queue.

## See also

- [`spl_autoload_unregister()`](thp:std.spl.spl_autoload_unregister)
- [`spl_autoload_functions()`](thp:std.spl.spl_autoload_functions)
- [`spl_autoload_call()`](thp:std.spl.spl_autoload_call)
- [`spl_autoload()`](thp:std.spl.spl_autoload)
- [PHP `spl_autoload_register()`](https://www.php.net/manual/en/function.spl-autoload-register.php)
