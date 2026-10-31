--TEST--
file object construction yields while opening a file
--FILE--
<?thp
scheduler_register_default();
$opener = async(function (): void {
    echo "A";
    $file = new SplFileObject("001-memory-binary-roundtrip.phpt");
    echo $file->fread(2);
    echo "C";
});
$other = async(function (): void { echo "B"; });
await($opener);
await($other);
--EXPECT--
AB--C
