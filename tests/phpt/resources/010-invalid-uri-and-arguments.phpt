--TEST--
Stream APIs distinguish invalid URIs, closed handles, and invalid values
--FILE--
<?thp

try {
    Streams::open("unknown://target", OpenMode::Read);
} catch (InvalidStreamUriException $error) {
    echo "invalid uri\n";
}

try {
    Streams::open("php://input", OpenMode::Read);
} catch (InvalidStreamUriException $error) {
    echo "legacy input uri\n";
}

$stream = MemoryStream::open();

try {
    $stream->read(-1);
} catch (ValueError $error) {
    echo "invalid length\n";
}

$stream->close();

try {
    $stream->seek(0);
} catch (ClosedStreamException $error) {
    echo "closed stream\n";
}
--EXPECT--
invalid uri
legacy input uri
invalid length
closed stream
