--TEST--
recursive caching iterator preserves recursive entries and caches visited keys
--FILE--
<?thp

class Leaf implements RecursiveIterator<int, int> {
    public int $position = 0;
    public function rewind(): void { $this->position = 0; }
    public function valid(): bool { return $this->position < 1; }
    public function key(): int { return 2; }
    public function value(): RecursiveEntry<int, int> { return new RecursiveEntry<int, int>(3); }
    public function advance(): void { $this->position = $this->position + 1; }
}

class Tree implements RecursiveIterator<int, int> {
    public int $position = 0;
    public function rewind(): void { $this->position = 0; }
    public function valid(): bool { return $this->position < 1; }
    public function key(): int { return 4; }
    public function value(): RecursiveEntry<int, int> { return new RecursiveEntry<int, int>(5, new Leaf()); }
    public function advance(): void { $this->position = $this->position + 1; }
}

$cache = new RecursiveCachingIterator<int, int>(new Tree(), RecursiveCachingIterator::FULL_CACHE);
$flat = new RecursiveIteratorIterator<int, int>($cache, RecursiveIteratorIterator::SELF_FIRST);
foreach ($flat as $value) { echo $value . "\n"; }
echo $cache->count() . "\n";
echo $cache->getCache()[4]->value() . "\n";
--EXPECT--
5
3
1
5
