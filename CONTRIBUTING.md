# Contributing to THP

Thank you for helping improve THP. Discussion and bug reports are coordinated
through the [THP repository on GitHub](https://github.com/thp-lang/thp).

THP is experimental. Language contracts live in `doc/content/language/`, and
standard-library contracts live in `doc/content/std/`. Treat them as proposals
unless a page explicitly states otherwise, and do not assume PHP compatibility
for behavior that THP has not defined.

## Before you start

The project is not yet accepting external code contributions. Apache-2.0
source contributions and the separate MIT terms for official binaries require
contribution terms reviewed by the project owner's lawyer before pull requests
can be accepted. Until those terms are published, please use issues and
Discussions for bug reports, design feedback, and proposals; maintainers will
close unsolicited external code pull requests without merging them.

- Search [issues](https://github.com/thp-lang/thp/issues) and pull requests to
  see whether the change is already being discussed.
- Open an issue before undertaking a substantial language, runtime, or
  architectural change. Describe the problem, proposed behavior, alternatives,
  and any unresolved decisions.
- Keep each contribution focused. Design changes, implementation,
  documentation, and tests for the same behavior should stay synchronized.

For a language change, define the observable behavior before implementing it:
accepted and rejected syntax, typing and inference rules, runtime and cleanup
semantics, diagnostics, and interactions with existing features.

## Maintainer workflow

1. Create a branch from `main` with a descriptive name.
2. Make the change and add or update tests and documentation as needed.
3. Run the checks relevant to the files you changed.
4. Commit with a short, imperative
   [Conventional Commit](https://www.conventionalcommits.org/) subject, such as
   `feat: define union type narrowing`.
5. Push the branch and open a pull request against `main`.

In the pull request, explain what changed and why, list the validation commands
you ran, link related issues, and call out unresolved questions, compatibility
effects, or diagnostic changes. Respond to review feedback with additional
commits; maintainers may squash commits when merging.

Do not submit material copied from another project unless its origin and
license are identified and compatible with the repository. Historical PHP
fixture provenance is documented in
[`tests/phpt/PHP_IMPORT.md`](tests/phpt/PHP_IMPORT.md); inactive imported
fixtures are not kept in the main tree.

## Changing documentation

The active documentation site is in `doc/`. Historical prototypes are not kept
in the main tree; never edit generated `doc/dist/` files.

Documentation sources are strict Markdown under `doc/content/`. Keep API facts
in YAML frontmatter and behavioral explanations and examples in the Markdown
body. Preserve stable URLs where possible, use consistent terminology, and
mark unstable contracts with `status: experimental`, including a notice about
what remains unimplemented. Every page also declares `availability:
implemented | partial | proposed`; this is separate from stability and must
agree with the implementation-status matrix. Examples should use small `<?thp`
programs and should identify unresolved behavior instead of silently inheriting
PHP semantics.

The documentation compiler requires Node.js 22 or newer and pnpm 10.14. From
`doc/`, install dependencies and validate a change with:

```sh
pnpm install --frozen-lockfile
pnpm format:check
pnpm lint
pnpm check
pnpm build
```

Use `pnpm dev` to rebuild and preview the site locally at
<http://localhost:4173>. Changes to responsive behavior, search, or other
browser-facing features should also run `pnpm test:browser`.

## Adding tests

Every implemented feature should include positive, negative, and boundary
coverage. Prefer unit tests in the relevant Rust crate for compiler or runtime
internals and PHPT fixtures under `tests/phpt/` for observable programs and
diagnostics.

PHPT fixture names use a numbered kebab-case form and contain the standard
sections:

```phpt
--TEST--
Describe one observable behavior
--FILE--
<?thp

echo "hello\n";
--EXPECT--
hello
```

Use `--EXPECTF--` when output contains variable text. Compile-error
expectations count as passing tests when the emitted diagnostic matches the
expectation. PHPT files contain THP source and must not be run through PHP.
Follow any more specific README in the fixture directory, such as
`tests/phpt/resources/README.md`.

Run one fixture or a directory from the repository root:

```sh
cargo run -p thp-test -- tests/phpt/path/to/001-example.phpt
cargo run -p thp-test -- tests/phpt/path/to/directory
```

For Rust changes, run the full validation suite when practical:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

If a full check is not practical, run the most relevant targeted tests and say
which checks were omitted in the pull request.
