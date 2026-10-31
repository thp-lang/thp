--TEST--
regex iterator scanning file lines yields to another coroutine
--FILE--
<?thp
scheduler_register_default();
$file = new SplFileObject("001-memory-binary-roundtrip.phpt");
$file->rewind();
$regex = new RegexIterator($file, '/^--FILE--/');
$reader = async(function () use ($regex): void {
    echo "A";
    if ($regex->valid()) { echo "C"; }
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABC
