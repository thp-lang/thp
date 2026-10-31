--TEST--
php://temp/maxmemory uses the typed temporary stream implementation
--FILE--
<?thp

$stream = Streams::open(
    "php://temp/maxmemory:3",
    OpenMode::ReadWrite,
);

if ($stream instanceof WritableStream) {
    $stream->writeAll("spill");
}
if ($stream instanceof SeekableStream) {
    $stream->seek(0);
}
if ($stream instanceof ReadableStream) {
    echo $stream->readAll() . "\n";
}
--EXPECT--
spill
