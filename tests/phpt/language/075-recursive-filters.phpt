--TEST--
recursive filters apply policies to child iterators during flattening
--FILE--
<?thp

class Leaves implements RecursiveIterator<int, int> {
    public int $position = 0;
    public function rewind(): void { $this->position = 0; }
    public function valid(): bool { return $this->position < 2; }
    public function key(): int { return $this->position; }
    public function value(): RecursiveEntry<int, int> { return new RecursiveEntry<int, int>($this->position + 2); }
    public function advance(): void { $this->position = $this->position + 1; }
}

class Root implements RecursiveIterator<int, int> {
    public int $position = 0;
    public function rewind(): void { $this->position = 0; }
    public function valid(): bool { return $this->position < 2; }
    public function key(): int { return $this->position; }
    public function value(): RecursiveEntry<int, int> {
        if ($this->position === 0) { return new RecursiveEntry<int, int>(8, new Leaves()); }
        return new RecursiveEntry<int, int>(9);
    }
    public function advance(): void { $this->position = $this->position + 1; }
}

class Even extends RecursiveFilterIterator<int, int> {
    public function accept(RecursiveEntry<int, int> $value, int $key): bool { return $value->value() % 2 === 0; }
}
class AtLeast extends RecursiveFilterIterator<int, int> {
    public int $minimum = 3;
    public function accept(RecursiveEntry<int, int> $value, int $key): bool { return $value->value() >= $this->minimum; }
}

$parents = new ParentIterator<int, int>(new Root());
foreach (new RecursiveIteratorIterator<int, int>($parents, RecursiveIteratorIterator::SELF_FIRST) as $value) { echo "p:" . $value . "\n"; }
$even = new Even(new Root());
foreach (new RecursiveIteratorIterator<int, int>($even, RecursiveIteratorIterator::SELF_FIRST) as $value) { echo "e:" . $value . "\n"; }
$atLeast = new AtLeast(new Root());
foreach (new RecursiveIteratorIterator<int, int>($atLeast, RecursiveIteratorIterator::SELF_FIRST) as $value) { echo "a:" . $value . "\n"; }
$callback = new RecursiveCallbackFilterIterator<int, int>(new Root(), function (RecursiveEntry<int, int> $entry, int $key): bool { return $entry->value() % 2 === 0; });
foreach (new RecursiveIteratorIterator<int, int>($callback, RecursiveIteratorIterator::SELF_FIRST) as $value) { echo "c:" . $value . "\n"; }
--EXPECT--
p:8
e:8
e:2
a:8
a:3
a:9
c:8
c:2
