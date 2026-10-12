--TEST--
Memory streams report capabilities and support writes, flushing, and relative seeks
--FILE--
<?thp
$stream = MemoryStream::open("abc");
var_dump($stream->isReadable());
var_dump($stream->isWritable());
var_dump($stream->isSeekable());
var_dump($stream->seek(-1, SeekFrom::End));
var_dump($stream->write("Z"));
$stream->flush();
var_dump($stream->seek(-2, SeekFrom::Current));
echo $stream->readAll() . "\n";
try {
    $stream->seek(-1);
} catch (ValueError $error) {
    echo "negative\n";
}
--EXPECT--
bool(true)
bool(true)
bool(true)
int(2)
int(1)
int(1)
bZ
negative
