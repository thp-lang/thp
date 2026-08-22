--TEST--
MemoryStream aliases share one cursor and seeking beyond end zero-fills on write
--FILE--
<?thp

$stream = MemoryStream::open("abcd");
$alias = $stream;

echo $stream->read(2) . "\n";
echo $alias->read(2) . "\n";

$alias->seek(6);
$alias->writeAll("z");
$stream->seek(0);

var_dump($stream->readAll() === "abcd\x00\x00z");
--EXPECT--
ab
cd
bool(true)
