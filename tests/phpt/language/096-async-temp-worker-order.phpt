--TEST--
temporary-file spill yields and aliased writes keep cursor order
--FILE--
<?thp
scheduler_register_default();
$stream = TempStream::open(0);
$first = async(function () use ($stream): void {
    echo "A";
    $stream->writeAll("A");
    echo "a";
});
$second = async(function () use ($stream): void {
    echo "B";
    $stream->writeAll("B");
    echo "b";
});
await($first);
await($second);
$stream->seek(0);
echo $stream->readAll();
--EXPECT--
ABabAB
