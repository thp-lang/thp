---
kind: guide
id: guide.modulesResolution
title: Modules and name resolution
summary: Understand interface extraction, export indexing, dependency graphs, and static project name resolution.
nav:
  section: internals
  order: 50
status: experimental
availability: implemented
notice: >-
  The current module system is statically discovered from project
  configuration and does not perform runtime loading.
---

`thp-modules` turns independently parsed project units into one deterministic
namespace. It deliberately does not depend on HIR, MIR, bytecode, or the VM.
This keeps file discovery and project names separate from language typing and
execution.

Consider two configured modules:

```thp
<?thp

namespace Example\Contracts;

interface Greeter
{

    public function greet(string $name): string;
}
```

```thp
<?thp

use Example\Contracts\Greeter;

function welcome(Greeter $greeter): string {
    return $greeter->greet("Ada");
}
```

## Interface extraction

Before type-checking bodies, the compiler extracts each non-entry module's
public declarations into a `ModuleInterface`. An interface records canonical
exports and an interface hash without requiring dependent modules to inspect
the implementation body.

All interfaces feed an `ExportIndex`. Duplicate canonical exports are reported
with both source locations. The index distinguishes type and function
declarations, matching THP's separate import kinds.

```console
thp inspect --project=. --emit=interfaces main.thp
```

The command lists each module, its namespace, interface hash, and exported
symbols.

## Graph construction and resolution

Imports become typed dependency edges in a `ModuleGraph`. The graph reports
unknown imports, computes strongly connected components for legal declaration
cycles, and provides deterministic dependency order. Executable top-level
statements are rejected outside the entry module.

```console
thp inspect --project=. --emit=module-graph main.thp
```

After the index exists, name resolution rewrites AST name references to their
canonical exports. Fully qualified references, namespace-relative references,
and explicit aliases therefore reach HIR with one resolved identity. Unknown or
ambiguous names remain source diagnostics and stop semantic lowering.

## Design choices compared with PHP

PHP namespaces organize names, but the runtime class and function tables still
grow as files are loaded. Class and function names are generally
case-insensitive, unqualified function calls in a namespace may fall back to a
global function, and dynamic names defer lookup until execution.

THP chooses case-sensitive canonical names, separate type and function import
tables, and no runtime global fallback. Interfaces are extracted first, so the
compiler can reject duplicate exports and unknown imports before checking any
body. Declaration SCCs permit mutual type and function references without
making initialization order observable; imported modules therefore contain
declarations rather than executable top-level setup. This is less permissive
than PHP's runtime symbol tables, but it gives type analysis one complete and
deterministic namespace.

Resolved AST units are combined and passed to
[type analysis and HIR lowering](thp:guide.typeAnalysisHir).
