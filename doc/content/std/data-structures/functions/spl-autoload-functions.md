---
kind: function
id: std.spl.spl_autoload_functions
title: spl_autoload_functions
summary: Returns the callbacks currently registered for autoloading.
name: spl_autoload_functions
order: 11
typeParameters: []
parameters: []
returns:
  type: vector<callable>
  description:
    A vector containing the registered callbacks in the order in which the
    runtime would invoke them. The vector is empty when no loaders are registered.
errors:
  - description:
      Inspecting an empty queue is not an error. This function is not expected to
      throw during normal operation; concrete runtime failure behavior is not yet
      finalized.
related: []
status: experimental
availability: proposed
notice:
  This PHP-inspired contract is proposed and is not yet implemented in this
  repository. Callable representation is not finalized.
version: "0.1"
module: data-structures
---

`spl_autoload_functions()` returns a snapshot of the process-wide autoload
queue.

## Behavior

The returned vector is a snapshot. Modifying it does not change the runtime's
autoload queue. Registering or unregistering a loader after this call does not
alter a snapshot that was already returned.

Callbacks added with the `prepend` option of
[`spl_autoload_register()`](thp:std.spl.spl_autoload_register) appear before older
entries. Each callable appears at most once because registration is idempotent.

## Example

Remove every current loader by iterating over a snapshot:

```thp
foreach (spl_autoload_functions() as $loader) {
    spl_autoload_unregister($loader);
}
```

Taking the snapshot first prevents queue mutation from skipping later entries.

## See also

- [`spl_autoload_register()`](thp:std.spl.spl_autoload_register)
- [`spl_autoload_unregister()`](thp:std.spl.spl_autoload_unregister)
- [`spl_autoload_call()`](thp:std.spl.spl_autoload_call)
- [PHP `spl_autoload_functions()`](https://www.php.net/manual/en/function.spl-autoload-functions.php)
