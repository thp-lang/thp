# Changelog

All notable user-visible changes to THP are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html). GitHub releases
remain marked as prereleases while THP is experimental.

## [Unreleased]

### Added

- Highlighted arrow closures and their parameters; added collection-function
  completion, hover, and signature help, plus callable-variable signature help.

## [0.5.0]

### Added

- Recognized native iterator generic arities in docblocks and aligned iterator
  diagnostics with the 0.5.0 compiler.

## [0.1.0]

### Added

- Published workspace diagnostics with full-document synchronization and
  unsaved source overlays.
- Added hover, completion, signature help, definition, references, safe rename,
  document and workspace symbols, semantic tokens, and full-document
  formatting.
- Added detached docblock validation and tooling-only local `@var` hints.
- Added independent signed release archives under `thp-lsp-v*` tags.
