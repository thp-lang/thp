--TEST--
readAll enforces its byte limit with IoException
--FILE--
<?thp

$stream = MemoryStream::open("12345");

try {
    $stream->readAll(4);
} catch (IoException $error) {
    echo "limit exceeded\n";
}

var_dump($stream->tell());
--EXPECT--
limit exceeded
int(0)
