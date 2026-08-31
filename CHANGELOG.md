# Changelog

All notable user-visible changes to THP are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html). GitHub releases
remain marked as prereleases while THP is experimental.

## [Unreleased]

### Changed

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
