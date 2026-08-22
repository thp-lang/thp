--TEST--
MemoryStream preserves arbitrary bytes and starts at position zero
--FILE--
<?thp

$bytes = "\x00\xffTHP";
$stream = MemoryStream::open($bytes);

var_dump($stream->tell());
var_dump($stream->readAll() === $bytes);
var_dump($stream->eof());
--EXPECT--
int(0)
bool(true)
bool(true)
