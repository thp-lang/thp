--TEST--
caching iterator prefetch suspends through a Stringable value
--FILE--
<?thp
class SlowCacheLabel {
    public int $calls = 0;
    public function __toString(): string {
        echo "A";
        $this->calls = $this->calls + 1;
        suspend();
        echo "C";
        return "label";
    }
}
scheduler_register_default();
$label = new SlowCacheLabel();
$values: vector<SlowCacheLabel> = [$label];
$cache = new CachingIterator(new VectorIterator($values), CachingIterator::CALL_TOSTRING);
$reader = async(function () use ($cache, $label): void {
    $cache->rewind();
    echo $label->calls;
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABC1
