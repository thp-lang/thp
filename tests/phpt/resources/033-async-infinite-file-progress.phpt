--TEST--
infinite iterator yields while obtaining and rewinding a file-backed cursor
--FILE--
<?thp
class FileCycle implements IteratorAggregate<int, string> {
    public function getIterator(): Traversable<int, string> {
        return new SplFileObject("001-memory-binary-roundtrip.phpt");
    }
}
scheduler_register_default();
$cycle = new InfiniteIterator(new FileCycle());
$reader = async(function () use ($cycle): void {
    echo "A";
    $cycle->rewind();
    if ($cycle->valid()) { echo "C"; }
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABC
