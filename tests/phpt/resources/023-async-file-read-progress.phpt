--TEST--
file reads yield to another coroutine before delivering their bytes
--FILE--
<?thp
scheduler_register_default();
$stream = Files::openRead("001-memory-binary-roundtrip.phpt");
$reader = async(function () use ($stream): void {
    echo "A";
    echo $stream->read(8);
    echo "a";
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
AB--TEST--a
