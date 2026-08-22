--TEST--
Closing a stream is idempotent and invalidates every alias
--FILE--
<?thp

$stream = MemoryStream::open("data");
$alias = $stream;

$stream->close();
$alias->close();

var_dump($stream->isClosed());
var_dump($alias->isClosed());

try {
    $alias->read(1);
} catch (ClosedStreamException $error) {
    echo "closed\n";
}
--EXPECT--
bool(true)
bool(true)
closed
