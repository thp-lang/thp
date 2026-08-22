---
kind: module
id: std.dataStructures
title: Data structures
summary: Generic containers, observer contracts, autoloading, and object utilities.
module: data-structures
order: 60
status: experimental
availability: proposed
notice:
  These PHP-inspired class contracts are not implemented. Generic types, iteration
  behavior, serialization, and concrete errors may change.
---

| Class                                                    | Purpose                                          |
| -------------------------------------------------------- | ------------------------------------------------ |
| [`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList) | Indexed double-ended linked sequence.            |
| [`SplStack`](thp:std.spl.SplStack)                       | Last-in, first-out stack.                        |
| [`SplQueue`](thp:std.spl.SplQueue)                       | First-in, first-out queue.                       |
| [`SplHeap`](thp:std.spl.SplHeap)                         | Base class for value-ordered heaps.              |
| [`SplMaxHeap`](thp:std.spl.SplMaxHeap)                   | Heap that extracts the greatest value first.     |
| [`SplMinHeap`](thp:std.spl.SplMinHeap)                   | Heap that extracts the smallest value first.     |
| [`SplPriorityQueue`](thp:std.spl.SplPriorityQueue)       | Heap ordered by explicit priorities.             |
| [`SplFixedArray`](thp:std.spl.SplFixedArray)             | Contiguous storage with an explicit size.        |
| [`ArrayObject`](thp:std.spl.ArrayObject)                 | Object wrapper around array-like storage.        |
| [`SplObjectStorage`](thp:std.spl.SplObjectStorage)       | Object-identity set with optional attached data. |

`SplFixedArray` and `ArrayObject` are retained only as PHP migration-analysis
placeholders. Their names are not accepted THP-native API names because THP has
separate `vector<T>` and `map<K, V>` types. A fixed-size sequence and typed map
wrapper require independently named contracts before implementation.

## See also

- [SPL reference](thp:std.dataStructures)
- [PHP SPL data structures](https://www.php.net/manual/en/spl.datastructures.php)

## Observer contracts

[`SplObserver`](thp:std.spl.SplObserver) receives notifications from
[`SplSubject`](thp:std.spl.SplSubject).

## Autoloading and object utilities

Autoloading lets an application defer loading a class, interface, or enum until
the declaration is first needed. THP maintains an ordered, process-wide queue of
loader callbacks, following PHP's SPL autoloading model.

## Registry functions

| Function                                                           | Description                                      |
| ------------------------------------------------------------------ | ------------------------------------------------ |
| [`spl_autoload_register()`](thp:std.spl.spl_autoload_register)     | Adds a callback to the autoload queue.           |
| [`spl_autoload_unregister()`](thp:std.spl.spl_autoload_unregister) | Removes a callback from the autoload queue.      |
| [`spl_autoload_functions()`](thp:std.spl.spl_autoload_functions)   | Returns the registered callbacks in queue order. |
| [`spl_autoload_call()`](thp:std.spl.spl_autoload_call)             | Runs the queue for a requested type name.        |

## Default loader

| Function                                                           | Description                                       |
| ------------------------------------------------------------------ | ------------------------------------------------- |
| [`spl_autoload()`](thp:std.spl.spl_autoload)                       | Searches include paths using configured suffixes. |
| [`spl_autoload_extensions()`](thp:std.spl.spl_autoload_extensions) | Reads or replaces the default loader's suffixes.  |

## Example

```thp
spl_autoload_register(function (string $class): void {
    $path = "./src/" . str_replace("\\", "/", $class) . ".thp";
    require $path;
});

$service = new App\Services\ReportService();
```

When `ReportService` is not already declared, the runtime passes its qualified
name to each registered loader until one defines it.

## Design background

The queue and callback signatures follow
[PHP's class autoloading model](https://www.php.net/manual/en/language.oop5.autoload.php).
THP preserves explicit collection typing when inspecting the queue and does not
guarantee that PHP autoloaders registered by dependencies share the THP
registry.

## See also

- [Standard PHP Library](thp:std.dataStructures)
- [PHP SPL autoloading functions](https://www.php.net/manual/en/ref.spl.php)
