---
kind: guide
id: guide.startProject
title: Start a project
summary: Create a multi-file THP project with configuration, namespaces, static discovery, and a repeatable workflow.
nav:
  section: learn
  order: 30
status: experimental
availability: implemented
notice: >-
  Project discovery and linking are implemented for command-line experiments.
  THP does not yet provide an init command, package manager, or web project template.
---

A THP project is an explicit root directory containing `thp.toml`. You choose
that root and an entry file when invoking the CLI. The configuration maps
namespace prefixes to source directories so the compiler can discover the
complete program before type checking and execution.

This tutorial builds a small project named `hello-project`. Create these files
with your editor; there is no `thp new` or `thp init` command yet.

```text
hello-project/
├── .gitignore
├── main.thp
├── thp.toml
└── src/
    ├── Contracts/
    │   └── Greeter.thp
    └── GreetingService.thp
```

## Define the project root

Save this minimal configuration as `hello-project/thp.toml`:

```toml
[autoload]
"App\\" = "src/"
```

The key is a case-sensitive THP namespace prefix. TOML requires the backslash
to be escaped, so `"App\\"` represents the THP prefix `App\`. The value is a
directory relative to the project root.

The mapping creates these module identities:

| File                        | Module ID               | Required namespace |
| --------------------------- | ----------------------- | ------------------ |
| `src/Contracts/Greeter.thp` | `App\Contracts\Greeter` | `App\Contracts`    |
| `src/GreetingService.thp`   | `App\GreetingService`   | `App`              |

A file's module ID comes from its configured prefix plus its relative path
without `.thp`. Its declared namespace must match the module ID's namespace;
the filename is the module's final segment, not a requirement that the file
export a declaration of exactly that name.

The entry file is different. `main.thp` is selected by the CLI, may sit outside
the mapped directories, and may contain executable top-level statements.

## Add a contract

Save this as `src/Contracts/Greeter.thp`:

```thp
<?thp

namespace App\Contracts;

interface Greeter
{

    public function greet(string $name): string;
}
```

Imported modules contain declarations only. They cannot print output, assign
top-level variables, or perform other initialization as a side effect of
loading.

## Add an implementation

Save this as `src/GreetingService.thp`:

```thp
<?thp

namespace App;

use App\Contracts\Greeter;

final class GreetingService implements Greeter
{

    private string $prefix;

    public function __construct(string $prefix)
    {
        $this->prefix = $prefix;
    }

    public function greet(string $name): string
    {
        return $this->prefix . ", " . $name . "!";
    }
}
```

`use` is a compile-time import. It does not execute a file or register a
runtime loader. Project discovery extracts exports from all mapped modules and
resolves the import before checking method bodies.

## Write the entry file

Save this as `main.thp`:

```thp
<?thp

use App\Contracts\Greeter;
use App\GreetingService;

$greeter: Greeter = new GreetingService("Hello");
$names: vector<string> = ["Ada", "Grace"];

foreach ($names as $name) {
    echo $greeter->greet($name) . "\n";
}
```

The entry may import project declarations and execute statements. Its explicit
`Greeter` type demonstrates interface dispatch and keeps the main program
independent of the concrete service type.

## Ignore checkout-local artifacts

Save this as `.gitignore`:

```gitignore
/thp.local.toml
/thp.lock
/.thp-cache/
```

`thp.local.toml` is for one checkout's overrides. `thp.lock` is generated from
configuration tooling and can contain merged local extension data. The cache
directory is a disposable path chosen in later CLI commands. Project templates
must ignore the local and lock files; THP never edits `.gitignore` for you.

## Check and run from the project root

Change into `hello-project` and use the current directory as the implicit
project root:

```sh
thp check main.thp
thp run main.thp
```

The result is:

```text
Hello, Ada!
Hello, Grace!
```

THP enters project mode only when the exact selected root contains `thp.toml`.
It never searches parent directories. From the parent directory, name the root
explicitly; the entry remains relative to that root:

```sh
thp check --project=hello-project main.thp
thp run --project=hello-project main.thp
```

An invalid import, mismatched namespace, duplicate export, type error in any
discovered module, or executable statement outside the entry prevents the
project from running.

## Inspect discovery and linking

Two inspect modes are specific to projects:

```sh
thp inspect --emit=interfaces main.thp
thp inspect --emit=module-graph main.thp
```

`interfaces` shows each module's exported surface and fingerprint.
`module-graph` shows resolved dependency edges, deterministic order, and legal
cyclic declaration groups. These views help diagnose a namespace or import
problem without waiting for execution.

## Add more source roots

One prefix may search an ordered list of directories, and a project may define
several prefixes:

```toml
[autoload]
"App\\" = ["src/", "generated/"]
"Vendor\\Package\\" = "vendor/package/src/"
```

This is static source discovery, not a package installation mechanism. You are
responsible for placing those source files in the project. Overlapping mappings
must not make one physical file represent multiple logical modules or make one
module ID resolve to different physical files.

Prefer a narrow application prefix over an empty global prefix. Keep reusable
modules declaration-only, make namespaces mirror paths, and put startup order
in explicit function calls from `main.thp`.

## Add runtime settings only when needed

The minimal `[autoload]` file is enough for this tutorial. Core runtime
settings have defaults and may be added by domain:

```toml
[autoload]
"App\\" = "src/"

[memory]
limit = "128M"

[request]
max_stack_depth = 512
max_open_handles = 256

[time]
max_execution = "30s"
```

The configuration parser implements these values, while current CLI project
execution does not yet enforce the configured limits. See
[Project configuration](thp:guide.projectConfiguration) for formats, target
overrides, local precedence, extensions, lock generation, and the exact
availability boundary.

## Warm a deployable project artifact

For repeated runs, choose a project-local cache directory:

```sh
thp run --opcache=.thp-cache main.thp
```

To prepare a frozen project, publish the module interfaces, module objects,
linked bytecode, and manifest before running:

```sh
thp cache-warm --opcache=.thp-cache main.thp
thp run --frozen --opcache=.thp-cache main.thp
```

Frozen execution verifies the warmed manifest and program without scanning the
mapped source directories. Re-run `cache-warm` after changing sources,
configuration, the entry selection, or compiler identity. OPcache is not a
dependency lock and does not make THP a production deployment platform.

## Suggested development workflow

1. Keep `main.thp` small and move reusable declarations into mapped modules.
2. Run `thp check main.thp` after edits.
3. Use `thp inspect --emit=module-graph` when imports do not resolve as
   expected.
4. Run the VM for the complete implemented language surface.
5. Add OPcache only when repeated compilation matters.
6. Check [Implementation status](thp:guide.implementationStatus) before
   depending on a language or standard-library feature.

The repository's `examples/project` directory is a tested, runnable version of
this shape with structured exception handling added.
