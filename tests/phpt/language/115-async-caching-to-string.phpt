--TEST--
direct caching iterator string conversion suspends through a user method
--FILE--
<?thp
class SlowDirectLabel {
    public function __toString(): string {
        echo "A";
        suspend();
        echo "C";
        return "label";
    }
}
scheduler_register_default();
$label = new SlowDirectLabel();
$values: vector<SlowDirectLabel> = [$label];
$cache = new CachingIterator(new VectorIterator($values), 0);
$cache->rewind();
$reader = async(function () use ($cache): void { echo $cache->__toString(); });
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABClabel
