--TEST--
limit iterator seek yields while repositioning a file-backed cursor
--FILE--
<?thp
scheduler_register_default();
$file = new SplFileObject("001-memory-binary-roundtrip.phpt");
$window = new LimitIterator($file, 0, 5);
$reader = async(function () use ($window): void {
    echo "A";
    $window->seek(1);
    echo "C";
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABC
