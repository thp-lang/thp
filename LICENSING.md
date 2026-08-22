# THP licensing

THP deliberately separates the license for repository source from the license
offered for specifically identified official binary releases. A license applies
only to the scope described below.

## Repository source

Unless a path carries its own license or notice, the original source code,
build scripts, documentation, examples, and THP-native tests in this repository
are licensed under the [Apache License 2.0](LICENSE). The required attribution
is recorded in [NOTICE](NOTICE).

Apache-2.0 applies to source distributions and to binaries built from the
source by parties other than the copyright holder. It includes its own patent,
notice, redistribution, and warranty terms.

## Official binary releases

An executable, library, runtime component, or other binary artifact is also
offered under the [MIT License](LICENSE-BINARY) only when an official THP
release provided by the copyright holder:

1. expressly identifies that artifact as an official binary release; and
2. includes `LICENSE-BINARY` in the release archive.

This additional MIT grant applies only to those identified official binary
artifacts. It does not relicense repository source, source archives, or binaries
built and distributed by another party. Official archives include both
`LICENSE-BINARY` and the Apache source `LICENSE` and `NOTICE` so the boundary is
visible at installation time.

## Programs and generated output

The THP copyright holder claims no copyright interest in a program merely
because it is written, compiled, interpreted, tested, or otherwise processed
with THP. A program author chooses the terms for their own THP source and for
output generated exclusively from that source. Distributions that include THP
runtime material must still comply with the terms applicable to that material.

## Third-party material

Dependencies and files carrying their own license or copyright notice remain
governed by those terms. The Apache source license and the separate MIT grant
for official binaries do not replace third-party obligations. Release archives
include a generated `THIRD-PARTY-NOTICES` inventory.

The historical PHP 8.5.6 fixture import is not distributed in the main tree.
Its provenance, upstream license link, checksum, and reimport procedure are
recorded in [`tests/phpt/PHP_IMPORT.md`](tests/phpt/PHP_IMPORT.md).
