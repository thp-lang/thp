--TEST--
php://memory uses the same behavior as the typed MemoryStream factory
--FILE--
<?thp

$stream = Streams::open("php://memory", OpenMode::ReadWrite);

if (
    $stream instanceof ReadableStream
    && $stream instanceof WritableStream
    && $stream instanceof SeekableStream
) {
    $stream->writeAll("uri");
    $stream->seek(0);
    echo $stream->readAll() . "\n";
    echo "read-write-seek\n";
}
--EXPECT--
uri
read-write-seek
