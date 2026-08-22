--TEST--
php://temp/maxmemory uses the typed temporary stream implementation
--FILE--
<?thp

$stream = Streams::open(
    "php://temp/maxmemory:3",
    OpenMode::ReadWrite,
);

if (
    $stream instanceof ReadableStream
    && $stream instanceof WritableStream
    && $stream instanceof SeekableStream
) {
    $stream->writeAll("spill");
    $stream->seek(0);
    echo $stream->readAll() . "\n";
}
--EXPECT--
spill
