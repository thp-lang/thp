--TEST--
recursive iterator hooks observe current stack and run once per boundary
--FILE--
<?thp

class LeafCursor implements RecursiveIterator<int, int> {
    public int $position = 0;
    public function rewind(): void { $this->position = 0; }
    public function valid(): bool { return $this->position < 1; }
    public function key(): int { return 1; }
    public function value(): RecursiveEntry<int, int> { return new RecursiveEntry<int, int>(2); }
    public function advance(): void { $this->position = $this->position + 1; }
}

class RootCursor implements RecursiveIterator<int, int> {
    public int $position = 0;
    public function rewind(): void { $this->position = 0; }
    public function valid(): bool { return $this->position < 1; }
    public function key(): int { return 3; }
    public function value(): RecursiveEntry<int, int> { return new RecursiveEntry<int, int>(4, new LeafCursor()); }
    public function advance(): void { $this->position = $this->position + 1; }
}

class Walk extends RecursiveIteratorIterator<int, int> {
    public int $marker = 7;
    public function beginIteration(): void { echo "begin\n"; }
    public function beginChildren(): void { echo "down:" . ($this->getSubIterator(0) !== null) . "\n"; }
    public function nextElement(): void { echo "next:" . $this->getDepth() . "\n"; }
    public function endChildren(): void { echo "up:" . ($this->getSubIterator(0) !== null) . "\n"; }
    public function endIteration(): void { echo "end\n"; }
}

$walk = new Walk(new RootCursor(), RecursiveIteratorIterator::SELF_FIRST);
foreach ($walk as $value) { echo "value:" . $value . "\n"; }
echo $walk->marker . "\n";
--EXPECT--
begin
next:0
value:4
down:true
next:1
value:2
up:true
end
7
