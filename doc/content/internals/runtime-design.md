---
kind: guide
id: guide.runtimeDesign
title: Runtime design
summary: Understand THP values, request ownership, collection sharing, cycle collection, streams, and host boundaries.
nav:
  section: internals
  order: 120
status: experimental
availability: implemented
notice: >-
  This page describes the current single-request-thread runtime and its
  implemented embedding boundaries.
---

The runtime owns data that exists while bytecode executes. Persistent compiler,
module, and cache state is kept outside the request heap, so one request's
values cannot become another request's mutable globals.

## Values and sharing

General VM slots are 16-byte `Value` records. Scalars fit directly; strings,
vectors, maps, objects, exceptions, and streams refer to request-managed heap
cells. Heap references use non-atomic reference counts because one request
executes on one VM thread.

Vectors and insertion-ordered maps use copy-on-write storage. Assigning a
collection is cheap, and the first mutation detaches shared storage:

```thp
<?thp
$left = [1, 2];
$right = $left;
$right[0] = 9;

var_dump($left);
var_dump($right);
```

`$left` retains its original elements. Objects deliberately behave differently:
aliases refer to the same property storage, so a property mutation is visible
through every alias.

## Heap accounting and cycles

`RequestHeap` accounts managed cells and payload capacity against the request's
heap limit. Reference counts reclaim ordinary acyclic values immediately. A
buffer records possible cycle roots; at request-safe points, a trial-deletion
pass reclaims unreachable vector, map, object, and exception cycles. Final
request teardown releases everything still owned by the request.

Exceptions are runtime objects with nominal class identity, message, code,
previous and suppressed failures, and THP stack frames. Runtime operations
return structured error kinds; the VM attaches source and call information.

## Input, output, streams, and limits

Request input is a bounded byte source with a shared cursor used by
`thp:/input`. Output is written synchronously to a host-provided sink. Stream
values hold shared cursor and close state plus a logical handle lease. Explicit
close, last-reference cleanup, or request teardown releases the native storage
and handle count.

Execution receives limits for instructions, elapsed time, managed heap, input
bytes, logical stack depth, and open handles. Relative filesystem operations
use a request-local base path rather than changing the host process directory.

## Embedding boundaries

`thp-embed` exposes safe engines, prepared projects, request/response values,
and a streaming SAPI trait. `thp-abi` wraps those facilities in ABI version 1
with opaque engine and prepared-project handles, size-versioned limit
structures, binary-safe owned buffers, synchronous callbacks, panic
containment, and explicit release functions. Rust object layouts never cross
the C boundary.

## Design choices compared with PHP

Both runtimes use reference counting, copy-on-write where appropriate, and a
cycle collector, but THP changes what those mechanisms represent:

- PHP's `array` combines sequence and map behavior with dynamic keys and values.
  THP splits it into `vector<T>` and `map<K, V>`, preserving element types and
  allowing direct typed VM operations.
- PHP integers follow the platform integer size. THP `int` is always a checked
  signed 64-bit value, so overflow and serialized behavior do not depend on the
  host word size.
- PHP supports general reference aliasing between variables. THP collection
  assignment produces copy-on-write values, while only object and handle
  aliases intentionally expose shared mutation.
- PHP request data is conventionally surfaced through superglobals and SAPI
  state. THP passes input, output, filesystem base, time, and resource limits
  through an explicit request context.
- PHP streams are selected through resource wrappers and mode strings. THP's
  stream design exposes capability interfaces and typed failures, with
  `using` providing deterministic cleanup in addition to last-reference and
  request-teardown safety nets.

The common memory-management ideas make the behavior familiar, while the typed
collections and explicit request boundary prioritize static checking and safe
embedding over PHP's dynamic uniformity.

Return to the [internals overview](thp:guide.internalsOverview) for the complete
compiler-to-runtime path.
