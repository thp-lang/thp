--TEST--
append iterator yields while rewinding a file-backed inner cursor
--FILE--
<?thp
scheduler_register_default();
$file = new SplFileObject("001-memory-binary-roundtrip.phpt");
$all = new AppendIterator<int, string>();
$all->append($file);
$reader = async(function () use ($all): void {
    echo "A";
    $all->rewind();
    if ($all->valid()) { echo "C"; }
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABC
