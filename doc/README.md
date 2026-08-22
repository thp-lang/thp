# THP documentation compiler

This directory contains the source and compiler for the static THP
documentation website. It requires Node 22 or newer and pnpm 10.14.

## Pipeline

```text
content/**/*.md + site.yaml
  → strict frontmatter and type-expression validation
  → symbol, inheritance, navigation, and link resolution
  → typed site model
  → TypeScript HTML views
  → dist/**/*.html
  → Pagefind static search index
```

Sources are Markdown, not MDX. Raw HTML is rejected. API facts belong in YAML
frontmatter; prose supplies behavior, examples, and notes. Link to stable
symbols with the `thp:` scheme:

```md
[dequeue()](thp:std.spl.SplQueue::dequeue)
```

Every callable has exactly one source page. An inherited member links to the
route of its declaring type, so `SplQueue::pop()` resolves to
`/std/data-structures/spl-doubly-linked-list/pop/`.

## Commands

```sh
pnpm dev            # rebuild on changes and serve http://localhost:4173
pnpm build          # render, validate, and create the Pagefind index
pnpm check          # type checking plus unit and integration tests
pnpm lint
pnpm format:check
pnpm test:browser   # responsive behavior, search, screenshots, and axe checks
```

GitHub Pages builds must apply the repository base path:

```sh
env SITE_URL=https://thp-lang.github.io BASE_PATH=/thp/ pnpm build
```

`dist/`, dependency folders, caches, coverage, browser traces, and screenshots
are generated and ignored.
