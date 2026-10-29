--TEST--
limit iterator rewind yields while advancing a file stream
--FILE--
<?thp
scheduler_register_default();
$file = new SplFileObject("001-memory-binary-roundtrip.phpt");
$window = new LimitIterator($file, 1, 1);
$reader = async(function () use ($window): void {
    echo "A";
    $window->rewind();
    if ($window->valid()) { echo "C"; }
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABC
