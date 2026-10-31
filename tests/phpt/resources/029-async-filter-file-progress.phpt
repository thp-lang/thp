--TEST--
filter iterator scans file lines without blocking another coroutine
--FILE--
<?thp
scheduler_register_default();
$file = new SplFileObject("001-memory-binary-roundtrip.phpt");
$file->rewind();
$filter = new CallbackFilterIterator($file, fn(string $value, int $key): bool => $key > 0);
$reader = async(function () use ($filter): void {
    echo "A";
    if ($filter->valid()) { echo "C"; }
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABC
