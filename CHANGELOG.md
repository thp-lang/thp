# Changelog

All notable user-visible changes to THP are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html). GitHub releases
remain marked as prereleases while THP is experimental.

## [Unreleased]

### Added

- lang: Implemented exact compiled-class dynamic construction with constructor
  defaults, named and variadic arguments, inheritance, visibility checks, and
  runtime generic-bound validation.
- std: Implemented `is_string`, `is_int`, `is_float`, `is_null`, `is_numeric`,
  `is_vector`, and `is_map`, with direct positive-branch narrowing for guards
  and `instanceof`. Narrowing preserves compatible union members and is
  invalidated by assignment.

### Changed

- bytecode: Bumped the bytecode schema from version 2 to 3 for dynamic
  construction metadata and checked narrowing; older cached or frozen bytecode
  must be regenerated.

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
