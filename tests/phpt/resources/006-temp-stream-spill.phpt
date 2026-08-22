--TEST--
TempStream preserves content and cursor when it spills to disk
--FILE--
<?thp

$stream = TempStream::open(4);
$stream->writeAll("abcdef");

var_dump($stream->tell());
$stream->seek(1);
echo $stream->read(4) . "\n";
$stream->seek(0);
var_dump($stream->readAll() === "abcdef");
--EXPECT--
int(6)
bcde
bool(true)
