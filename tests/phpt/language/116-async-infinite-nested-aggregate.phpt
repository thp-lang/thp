--TEST--
infinite iterator resolves nested aggregates across suspension
--FILE--
<?thp
class SlowNestedSource implements IteratorAggregate<int, int> {
    public function getIterator(): Traversable<int, int> {
        echo "A";
        suspend();
        return new VectorIterator([4]);
    }
}
class OuterSource implements IteratorAggregate<int, int> {
    public function getIterator(): Traversable<int, int> { return new SlowNestedSource(); }
}
scheduler_register_default();
$reader = async(function (): void {
    $cycle = new InfiniteIterator(new OuterSource());
    $cycle->rewind();
    echo $cycle->value();
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
AB4
