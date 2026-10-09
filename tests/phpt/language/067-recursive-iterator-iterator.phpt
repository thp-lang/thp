--TEST--
recursive iterator iterator traverses all modes and depth limits
--FILE--
<?thp

class Leaves implements RecursiveIterator<int, int> {
    public int $position = 0;
    public function rewind(): void { $this->position = 0; }
    public function valid(): bool { return $this->position < 2; }
    public function key(): int { return $this->position + 10; }
    public function value(): RecursiveEntry<int, int> { return new RecursiveEntry<int, int>($this->position + 20); }
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

foreach ([RecursiveIteratorIterator::LEAVES_ONLY, RecursiveIteratorIterator::SELF_FIRST, RecursiveIteratorIterator::CHILD_FIRST] as $mode) {
    $flat = new RecursiveIteratorIterator<int, int>(new Root(), $mode);
    foreach ($flat as $key => $value) { echo $flat->getDepth() . ":" . $key . ":" . $value . "\n"; }
    echo "end:" . $flat->getDepth() . "\n";
}
$flat = new RecursiveIteratorIterator<int, int>(new Root(), RecursiveIteratorIterator::LEAVES_ONLY);
$flat->setMaxDepth(0);
foreach ($flat as $value) { echo "limited:" . $value . "\n"; }
echo $flat->getMaxDepth() . "\n";
--EXPECT--
1:10:20
1:11:21
end:1
0:7:8
1:10:20
1:11:21
end:1
1:10:20
1:11:21
0:7:8
end:0
limited:8
0
