--TEST--
php://memory uses the same behavior as the typed MemoryStream factory
--FILE--
<?thp

$stream = Streams::open("php://memory", OpenMode::ReadWrite);

if ($stream instanceof WritableStream) {
    $stream->writeAll("uri");
}
if ($stream instanceof SeekableStream) {
    $stream->seek(0);
}
if ($stream instanceof ReadableStream) {
    echo $stream->readAll() . "\n";
    echo "read-write-seek\n";
}
--EXPECT--
uri
read-write-seek
