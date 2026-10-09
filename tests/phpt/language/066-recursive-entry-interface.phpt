--TEST--
recursive iterator interfaces yield immutable entries with child cursors
--FILE--
<?thp

class Leaves implements RecursiveIterator<int, int> {
    public int $position = 0;
    public function rewind(): void { $this->position = 0; }
    public function valid(): bool { return $this->position < 1; }
    public function key(): int { return 0; }
    public function value(): RecursiveEntry<int, int> { return new RecursiveEntry<int, int>(9); }
    public function advance(): void { $this->position = $this->position + 1; }
}

class Root implements RecursiveIterator<int, int> {
    public int $position = 0;
    public function rewind(): void { $this->position = 0; }
    public function valid(): bool { return $this->position < 1; }
    public function key(): int { return 7; }
    public function value(): RecursiveEntry<int, int> { return new RecursiveEntry<int, int>(8, new Leaves()); }
    public function advance(): void { $this->position = $this->position + 1; }
}

foreach (new Root() as $key => $entry) {
    echo $key . ":" . $entry->value() . "\n";
    echo $entry->children() !== null;
    echo "\n";
}
$children = new Leaves();
foreach ($children as $child) { echo $child->value() . "\n"; }
$leaf = new RecursiveEntry<int, int>(1);
echo $leaf->children() === null;
--EXPECT--
7:8
true
9
true
