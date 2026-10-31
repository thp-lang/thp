--TEST--
caching iterator calls string conversion while visiting entries when requested
--FILE--
<?thp
class Label {
    public int $calls = 0;
    public function __toString(): string {
        $this->calls = $this->calls + 1;
        return "label";
    }
}
$label = new Label();
$values: vector<Label> = [$label];
$cache = new CachingIterator(new VectorIterator($values), CachingIterator::CALL_TOSTRING);
$cache->rewind();
echo $label->calls === 1;
$plain = new CachingIterator(new VectorIterator($values), 0);
$plain->rewind();
echo $label->calls === 1;
--EXPECT--
truetrue
