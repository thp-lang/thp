---
kind: guide
id: guide.languageNamespaces
title: Namespaces
summary: Describes THP static modules, qualified names, imports, and source discovery.
nav:
  section: language
  order: 110
status: experimental
availability: partial
notice: >-
  The initial compiler implements static syntax, project discovery, alias
  resolution, module graphs, frozen linked programs, and prepared projects.
  Relocatable per-module code generation and fine-grained incremental
  invalidation remain experimental work.
---

Namespaces organize declarations and prevent unrelated packages from claiming
the same short name. Their PHP-shaped syntax is a compile-time naming facility,
not a runtime loader. THP discovers source files from project configuration and
links one verified program before execution.

```thp
<?thp

namespace App\Service;

use Vendor\Package\Client;
use Vendor\Package\Client as PackageClient;
use function Vendor\Package\makeClient;
```

## File grammar

A file contains zero or one semicolon-style `namespace` declaration immediately
after `<?thp`. Omitting it selects the global namespace. Bracketed declarations,
multiple declarations, and a declaration after any other construct are errors.

Imports follow the namespace declaration and precede every top-level
declaration or statement. Type imports and function imports have separate alias
tables. Duplicate aliases in one table are errors.

THP rejects comma/group imports, `use const`, and relative
`namespace\Name` references. Fully qualified references begin with `\`.

## Static resolution

- An unqualified name contains one segment, such as `Client`.
- A qualified name contains namespace segments, such as `Http\Client`.
- A fully qualified name begins from the global namespace and is absolute.
- For qualified names, an imported first segment is replaced; otherwise the
  current namespace is prepended.
- An unqualified type uses its type alias or the current namespace.
- An unqualified function uses its function alias, then a function in the
  current namespace, then the static THP prelude.

Resolution is case-sensitive and complete at compilation. Unknown imported
targets are errors. THP does not perform dynamic-name lookup or runtime global
fallback.

## Source discovery

Projects map namespace prefixes to one or more ordered directories:

```toml
[autoload]
"App\\" = "src/"
"Vendor\\Package\\" = ["vendor/package/src/"]
```

Relative directories resolve from the selected project root. Explicit absolute
and symlinked directories are permitted. Discovery canonicalizes physical
files, sorts logical module IDs, and rejects one logical ID backed by different
files or one physical file mapped to different IDs.

A `.thp` file is one module. Its logical ID is the configured prefix followed
by its relative path without `.thp`. For example,
`src/Service/Client.thp` under `App\` is module
`App\Service\Client` and must declare `namespace App\Service;`. Several files
may export declarations into the same namespace. An autoloaded file may use the
global namespace only under an empty prefix.

The project root is the current directory unless `--project=DIR` is provided.
THP never searches ancestor directories. If that exact root has no `thp.toml`,
the command compiles only the named file.

## Linking and initialization

All top-level functions, classes, interfaces, and traits are exported.
Imported modules cannot contain executable top-level statements; initialization
belongs in an explicit function called by the entry file.

Mutual function/type references and cross-file recursion form legal cyclic
declaration groups. Inheritance, interface-extension, and trait-composition
cycles remain type errors.

`thp inspect --emit=module-graph` prints deterministic dependency edges and
cyclic groups. `--emit=interfaces` prints canonical exports and interface
fingerprints. `thp cache-warm` publishes module interfaces, module objects, a
verified linked program, and its frozen manifest. `thp run --frozen` reads the
manifest and linked program without enumerating source directories.

## See also

- [Classes and objects](thp:guide.languageClassesAndObjects)
- [Functions](thp:guide.languageFunctions)
- [Constants](thp:guide.languageConstants)
