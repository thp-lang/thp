---
kind: function
id: std.spl.spl_autoload_unregister
title: spl_autoload_unregister
summary: Removes a callback from the autoload queue.
name: spl_autoload_unregister
order: 13
typeParameters: []
parameters:
  - name: callback
    type: callable
    description: The same callable value supplied at registration.
returns:
  type: bool
  description: true when the matching callback is removed; otherwise false.
errors:
  - description:
      Passing a value that is not callable is a type error. A callable that is
      valid but absent from the queue produces false rather than an error.
related: []
status: experimental
availability: proposed
notice:
  This PHP-inspired contract is proposed and is not yet implemented in this
  repository. Callable identity rules may change.
version: "0.1"
module: data-structures
---

`spl_autoload_unregister()` removes a previously registered loader callback.

## Behavior

The function compares callable identity, not merely equivalent callback
behavior. Keep the original closure or callable object if it will later be
unregistered.

Because registration is idempotent, at most one matching queue entry exists. Removing it leaves the queue without that callback; removing the final callback disables automatic loading until another callback is registered.

The function does not invoke the callback and does not unload declarations that
were already loaded.

## Example

```thp
$loader = function (string $class): void {
    require "./generated/" . str_replace("\\", "/", $class) . ".thp";
};

spl_autoload_register($loader);

// Generated declarations are no longer searched after this point.
$removed = spl_autoload_unregister($loader);
```

`$removed` is `true` for the registration created in the example.

## See also

- [`spl_autoload_register()`](thp:std.spl.spl_autoload_register)
- [`spl_autoload_functions()`](thp:std.spl.spl_autoload_functions)
- [`spl_autoload_call()`](thp:std.spl.spl_autoload_call)
- [PHP `spl_autoload_unregister()`](https://www.php.net/manual/en/function.spl-autoload-unregister.php)
