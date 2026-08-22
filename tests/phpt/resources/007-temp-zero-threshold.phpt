--TEST--
TempStream accepts a zero memory threshold and rejects a negative threshold
--FILE--
<?thp

$stream = TempStream::open(0);
$stream->writeAll("disk-backed");
$stream->seek(0);
echo $stream->readAll() . "\n";

try {
    TempStream::open(-1);
} catch (ValueError $error) {
    echo "negative threshold\n";
}
--EXPECT--
disk-backed
negative threshold
