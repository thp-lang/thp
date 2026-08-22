---
kind: module
id: std.streams
title: Streams
summary: Typed native handles, deterministic cleanup, and capability-based byte streams.
module: streams
order: 60
status: experimental
availability: partial
notice:
  The handle model, stream interfaces, factories, modes, and exceptions are a
  design proposal. They are not implemented in this repository and may change as runtime
  integration proceeds.
---

THP does not expose PHP's generic `resource` type. External state is represented
by nominal, opaque native objects such as `MemoryStream` and file-stream
classes. Their types describe what operations are available, while the runtime
keeps ownership and cleanup predictable.

## PHP baseline

PHP resources are opaque handles created by extension functions. Copies share
the same reference-counted resource, explicit close functions invalidate that
shared resource, and PHP releases most resources when their last reference is
gone. Persistent connections are an exception to automatic release.

PHP streams put files, URLs, memory buffers, temporary storage, and filters
behind the same procedural functions and `scheme://target` naming convention.
This is flexible, but a parameter typed only as `resource` does not say whether
it is a stream, database result, process, or another handle kind. Supported
stream operations are also commonly discovered only at runtime.

Modern PHP has moved many extension resources to nominal internal objects,
including cURL, socket, GD, OpenSSL, FTP, LDAP, and PostgreSQL handles. THP
adopts that typed direction without retaining a generic resource escape hatch.

## Native handle ownership

A native handle is a small reference to one shared runtime cell. The cell
contains:

- a non-atomic reference count;
- a nominal native type identifier;
- an open or closed state;
- the native payload;
- the payload's cleanup operation.

Assigning or passing a handle increments the cell's reference count. Releasing
a reference decrements it. The transition to zero closes an open payload and
destroys the cell. THP initially runs handles within one VM thread, so ordinary
copies do not require atomic operations.

Built-in handle classes are sealed, cannot be directly constructed, and do not
have dynamic properties. Their operations use direct native dispatch rather
than user-overridable destructors.

### Shared close state

All aliases observe the same state. `close()` closes the payload immediately
and is idempotent. A later operation through any alias throws
[`ClosedStreamException`](thp:std.streams.ClosedStreamException).

```thp
<?thp

$stream = MemoryStream::open();
$alias = $stream;

$stream->close();
$alias->close(); // No effect.
$alias->read(1); // Throws ClosedStreamException.
```

Last-reference cleanup is a non-throwing safety net. Programs should use
`using`, `close()`, or a successful writer `flush()` when the timing or outcome
of release matters. The runtime also closes remaining native handles during VM
shutdown.

## Deterministic cleanup

`using` owns cleanup for a block:

```thp
<?thp

using ($stream = MemoryStream::open()) {
    $stream->writeAll("report\n");
    $stream->seek(0);
    echo $stream->readAll();
}
```

The initializer must produce a `Closeable`. The binding is visible only inside
the block. Its `close()` method runs exactly once when control leaves the block
by fallthrough, `return`, `break`, `continue`, or an exception. Multiple
resources use nested `using` blocks and therefore close in reverse nesting
order.

An alias may escape the block, but the handle is closed when the block ends.
Using the escaped alias then throws the appropriate closed-handle exception.

If the body completes normally and `close()` throws, the close exception
propagates. If the body and `close()` both throw, the body exception remains
primary and the close exception is appended to its suppressed exceptions.

```thp
interface Closeable
{

    public function close(): void;
    public function isClosed(): bool;
}
```

Application classes may implement `Closeable`. Native handle classes implement
it directly in the runtime.

## Stream capabilities

The base `Stream` interface provides lifetime and capability inspection.
Capability interfaces add only the operations they support.

```thp
interface Stream extends Closeable
{

    public function isReadable(): bool;
    public function isWritable(): bool;
    public function isSeekable(): bool;
}

interface ReadableStream extends Stream
{

    public function read(int $length): string;
    public function readAll(?int $limit = null): string;
    public function eof(): bool;
}

interface WritableStream extends Stream
{

    public function write(string $data): int;
    public function writeAll(string $data): void;
    public function flush(): void;
}

interface SeekableStream extends Stream
{

    public function seek(
        int $offset,
        SeekFrom $from = SeekFrom::Start,
    ): int;
    public function tell(): int;
}

enum SeekFrom
{

    case Start;
    case Current;
    case End;
}
```

`read(0)` returns an empty string. A negative length or limit throws
`ValueError`. `read()` returns an empty string at end of stream; `eof()`
distinguishes that condition from an empty read request. `readAll()` reads until
EOF. If the remaining data exceeds a non-null limit, it throws `IoException`
without consuming data or changing the cursor.

`write()` may accept only a prefix and returns its byte count. `writeAll()`
continues until all bytes are written or an `IoException` occurs. `flush()`
reports buffered-write failures.

`seek()` returns the new absolute byte position. A negative resulting position
throws `ValueError`. Seeking beyond the current end is allowed for memory,
temporary, and regular-file streams; a subsequent write fills the gap with
zero bytes.

## Typed factories

Typed factories are the primary API. They return concrete types whose
interfaces expose the supported operations without mode strings or runtime
casts.

```thp
final class MemoryStream
    implements ReadableStream, WritableStream, SeekableStream
{
    public static function open(string $initial = ""): MemoryStream;
}

final class TempStream
    implements ReadableStream, WritableStream, SeekableStream
{
    public static function open(
        int $maxMemoryBytes = 2097152,
    ): TempStream;
}

final class Files
{

    public static function openRead(string $path): ReadableFileStream;
    public static function openWrite(
        string $path,
        WriteMode $mode = WriteMode::Truncate,
    ): WritableFileStream;
    public static function openReadWrite(
        string $path,
        WriteMode $mode = WriteMode::OpenExisting,
    ): ReadWriteFileStream;
}

enum WriteMode
{

    case OpenExisting;
    case Truncate;
    case Append;
    case Create;
    case CreateExclusive;
}
```

| `WriteMode`       | Existing path                   | Missing path | Initial position | Write position |
| ----------------- | ------------------------------- | ------------ | ---------------- | -------------- |
| `OpenExisting`    | Preserve contents               | Fail         | Start            | At cursor      |
| `Truncate`        | Truncate to zero bytes          | Create       | Start            | At cursor      |
| `Append`          | Preserve contents               | Create       | End              | Always at end  |
| `Create`          | Preserve contents               | Create       | Start            | At cursor      |
| `CreateExclusive` | Fail with `OpenStreamException` | Create       | Start            | At cursor      |

`MemoryStream` stores its contents in a geometrically growing memory buffer.
Its initial cursor is zero, including when initial contents are provided.

`TempStream` behaves like `MemoryStream` until a write would grow it beyond
`maxMemoryBytes`. It then creates an anonymous temporary file, copies existing
data once, preserves the cursor, and continues there. A threshold of zero uses
a temporary file from the first write. Negative thresholds throw `ValueError`.
The selected backend is deliberately not observable through the public API.

File factories separate read, write, and read-write access in their return
types. Opening and I/O failures use typed exceptions instead of `false`.

## URI compatibility bridge

Dynamic names use a compatibility API:

```thp
final class Streams
{

    public static function open(
        string $uri,
        OpenMode $mode,
    ): Stream;
}

enum OpenMode
{

    case Read;
    case Write;
    case Append;
    case ReadWrite;
    case ReadWriteTruncate;
    case ReadWriteAppend;
    case CreateExclusive;
}
```

The modes correspond to PHP's familiar mode strings:

| `OpenMode`          | PHP mode | Requested capabilities             |
| ------------------- | -------- | ---------------------------------- |
| `Read`              | `r`      | Read and seek                      |
| `Write`             | `w`      | Write and seek; truncate or create |
| `Append`            | `a`      | Write and seek; create if missing  |
| `ReadWrite`         | `r+`     | Read, write, and seek              |
| `ReadWriteTruncate` | `w+`     | Read, write, and seek; truncate    |
| `ReadWriteAppend`   | `a+`     | Read, write, and seek; append      |
| `CreateExclusive`   | `x+`     | Read, write, and seek; new target  |

The first version recognizes local file paths, `file://`, `php://memory`, and
`php://temp/maxmemory:N`. It also recognizes the shared, read-only request
stream `thp:/input`. Unknown schemes and malformed wrapper options throw
[`InvalidStreamUriException`](thp:std.streams.InvalidStreamUriException).
User-registered wrappers, network schemes, standard descriptors, output
wrappers, and `php://filter` are reserved for later proposals.

The returned object's implemented capability interfaces reflect the requested
mode. If a scheme cannot provide the requested capability,
`UnsupportedStreamOperationException` is thrown during opening.

Because a URI may be computed at runtime, `Streams::open()` returns only
`Stream`. Use `instanceof` to narrow the result before capability-specific
operations:

```thp
<?thp

$stream = Streams::open("php://memory", OpenMode::ReadWrite);

if (!($stream instanceof WritableStream)) {
    throw new UnsupportedStreamOperationException("stream is not writable");
}

$stream->writeAll("payload");
```

Typed factories and the URI bridge use the same underlying stream
implementations. URI parsing is paid only when a stream is opened.

### Request I/O

`Streams::open("thp:/input", OpenMode::Read)` returns one shared body stream
and cursor per request. Other modes throw
`UnsupportedStreamOperationException`. Body size and input time are request
limits; source/module loading and ordinary file reads are not request input.

Program output is written synchronously to the host sink and has no total-size
quota. Captured output is an explicit embedding and test-runner convenience.

## Binary strings

For PHP-inspired stream I/O, THP `string` values are arbitrary byte
sequences, not guaranteed UTF-8 text. String length, offsets, comparisons,
concatenation, stream positions, and I/O counts operate in bytes.

UTF-8-aware APIs validate their input explicitly and throw `ValueError` for an
invalid sequence unless that API documents a replacement policy. Stream reads
perform no encoding validation or transcoding.

This representation lets a read return one reference-counted byte allocation
and lets writes borrow the existing string while copying only into storage that
must retain the data.

## Failures

Stream failures derive from `Exception`:

```thp
class IoException extends Exception
{
}

class OpenStreamException extends IoException
{
}

class ClosedStreamException extends IoException
{
}

class UnsupportedStreamOperationException extends IoException
{
}

class InvalidStreamUriException extends OpenStreamException
{
}
```

[`OpenStreamException`](thp:std.streams.OpenStreamException) records
the requested path or URI and the platform error code when available.
`InvalidStreamUriException` is used before I/O begins. `ClosedStreamException`
reports access after shared close.
[`UnsupportedStreamOperationException`](thp:std.streams.UnsupportedStreamOperationException)
reports a capability unavailable on a dynamically opened stream. Invalid
numeric arguments use `ValueError`.

## Performance contract

The proposal requires:

- one shared-cell allocation per built-in handle;
- non-atomic reference counting while handles are VM-thread-confined;
- direct native dispatch for typed factories;
- amortized constant-time memory-stream writes;
- no UTF-8 validation during binary I/O;
- no more than one memory-to-file spill per temporary stream;
- capacity-aware `readAll()` allocation when the remaining size is known;
- direct host output writes without a request-sized capture buffer.

PHPT fixtures define observable semantics. Runtime implementation must add
microbenchmarks for factory and URI opening, handle copies, memory-stream
throughput, and temporary-stream spill behavior before performance claims are
treated as stable.

## See also

- [Language resources and streams](thp:guide.languageResourcesAndStreams)
- [Types](thp:guide.languageTypes)
- [Control structures](thp:guide.languageControlStructures)
- [Exceptions](thp:guide.languageExceptions)
- [PHP resources](https://www.php.net/manual/en/language.types.resource.php)
- [PHP streams](https://www.php.net/manual/en/book.stream.php)
- [PHP `php://` wrappers](https://www.php.net/manual/en/wrappers.php.php)
- [PHP 8 resource-to-object migration](https://www.php.net/manual/en/migration80.incompatible.php)
