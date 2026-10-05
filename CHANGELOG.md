# Changelog

All notable user-visible changes to THP are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html). GitHub releases
remain marked as prereleases while THP is experimental.

## [Unreleased]

## [0.8.0] - 2026-10-05

### Added

- std: Implemented foundational contracts for `Option`, `Countable`, `MapAccess`, `Stringable`, throwable traces, the exception hierarchy, and `serialize()`/`unserialize()`.
- tooling: Added cross-platform release installers with archive checksum verification.

### Changed

- runtime: Advanced the bytecode schema to version 7 for foundational standard-library values and serialization.

## [0.7.0] - 2026-10-03

### Added

- lang: Added lazy, one-shot `Generator<K, V>` functions with automatic and explicit keys, final return values, and typed `Iterator<K, V>` traversal.
- runtime: Added suspension-aware `finally` and `using` cleanup on resume and explicit close, with exceptions propagating from the resume that executes them.
- runtime: Advanced the bytecode schema to version 6 for generator frames and yield terminators.

## [0.6.0] - 2026-10-03

### Added

- lang: Added typed `callable<P..., R>` values, explicit value captures in block closures, implicit value captures in arrow closures, and checked callable invocation.
- std: Added vector and map transformations, `iterator_apply()`, `FilterIterator`, and `CallbackFilterIterator`.
- runtime: Advanced the bytecode schema to version 5 for closure values and verified dynamic calls.

## [0.5.0] - 2026-10-03

### Added

- lang: Added rewindable native `VectorIterator<T>`, `MapIterator<K, V>`,
  `EmptyIterator<K, V>`, and `IteratorIterator<K, V>` cursors, with typed
  collection conversion at iterator boundaries.
- std: Added consuming `iterator_count()`, `iterator_to_vector()`, and
  `iterator_to_map()` functions.
- runtime: Advanced the bytecode schema to version 4 for native iterator
  builtins and verification.
- tooling: Updated `thp-lsp` to 0.5.0 with native iterator type arities and
  compiler diagnostic parity.

## [0.4.0] - 2026-10-03

### Added

- lang: Implemented keyed and value-only `foreach` over invariant
  `Traversable<K, V>` objects, including nested aggregate delegation, virtual
  iterator dispatch, loop transfers, and cleanup-preserving failures.

## [0.3.0] - 2026-09-21

### Added

- lang: Implemented reified generic classes and single-parent generic
  interfaces with invariant type arguments, bounds, inference, and checked
  substitutions.
- lang: Implemented exact compiled-class dynamic construction with constructor
  defaults, named and variadic arguments, inheritance, visibility checks, and
  runtime generic-bound validation.
- lang: Added deterministic reflection over linked classes, functions, methods,
  parameters, properties, and types, including checked access, invocation, and
  construction.
- std: Implemented `is_string`, `is_int`, `is_float`, `is_null`, `is_numeric`,
  `is_vector`, and `is_map`, with direct positive-branch narrowing for guards
  and `instanceof`. Narrowing preserves compatible union members and is
  invalidated by assignment.
- config: Added installed-package source discovery and deterministic `thp.lock`
  generation and loading through the new `thp lock` command.
- std: Implemented the invariant `Traversable<K, V>`, `Iterator<K, V>`, and
  `IteratorAggregate<K, V>` interfaces; iterator-object `foreach` remains
  proposed.
- runtime: Implemented `TypeError` and its `ArgumentCountError` subclass for
  dynamic argument binding failures.
- tooling: Added the independently versioned `thp-lsp` 0.1.0 stdio server with
  workspace diagnostics, navigation, rename, symbols, semantic tokens,
  source-preserving formatting, detached docblocks, and local `@var` hints.

### Changed

- bytecode: Bumped the bytecode schema from version 1 to 3 for reified generics,
  dynamic construction, checked narrowing, and reflection metadata; older
  cached or frozen bytecode must be regenerated.
- config: Extended the experimental version-1 lock layout with package and
  autoload records; locks generated with the previous layout must be
  regenerated.
- lang: Limited typed constant defaults to 128 nested collection levels.

### Removed

- doc: Removed the proposed `spl_autoload_*` callback APIs; THP uses configured
  static project and installed-package discovery instead.

### Fixed

- lang: Invalidated guarded type refinements before loop reads when a `while`,
  `for`, or `foreach` path can assign the refined local.

## [0.2.0] - 2026-09-02

### Changed

- deps: Update blake3, toml, cranelift-* and criterion packages
- doc: Reconciled language, iterator, collection, and stream availability with the
  executable compiler and VM, using symbol-level implementation matrices.
- doc: Defined the proposed invariant `Traversable<K, V>`, `Iterator<K, V>`, and
  `IteratorAggregate<K, V>` contracts and PHP-aligned `foreach` dispatch,
  rewind, mutation, and cleanup behavior.
- doc: Distinguished implemented, non-consuming
  `count(string|vector<T>|map<K, V>)` from proposed, consuming
  `iterator_count(Iterator<K, V>)`.
- doc: Kept iterator adapters, conversions, and native collection transformations
  explicitly proposed for later releases.

## [0.1.0] - 2026-08-22

Initial experimental release of the standalone THP compiler, bytecode VM,
OPcache, baseline JIT, module system, embedding API, and version-one C ABI.
