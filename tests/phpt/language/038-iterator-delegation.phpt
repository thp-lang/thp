--TEST--
IteratorIterator forwards cursor calls and consuming map conversion preserves first key position
--FILE--
<?thp

class Repeated implements Iterator<string, int> {
    public int $position = 0;
    public function rewind(): void { echo "rewind\n"; $this->position = 0; }
    public function valid(): bool { echo "valid:" . $this->position . "\n"; return $this->position < 3; }
    public function key(): string {
        echo "key:" . $this->position . "\n";
        if ($this->position === 1) { return "b"; }
        return "a";
    }
    public function value(): int { echo "value:" . $this->position . "\n"; return $this->position + 1; }
    public function advance(): void { echo "advance:" . $this->position . "\n"; $this->position = $this->position + 1; }
}

$inner = new Repeated();
$outer = new IteratorIterator($inner);
echo ($outer->getInnerIterator() === $inner) . "\n";
$outer->advance();
$map = iterator_to_map($outer);
foreach ($map as $key => $value) { echo $key . ":" . $value . "\n"; }
$repeatedMap = iterator_to_map(new Repeated());
foreach ($repeatedMap as $key => $value) { echo $key . ":" . $value . "\n"; }
$outer->rewind();
echo $inner->key() . ":" . $outer->value() . "\n";

class Failing implements Iterator<int, int> {
    public function rewind(): void {}
    public function valid(): bool { throw new Exception("delegated"); }
    public function key(): int { return 0; }
    public function value(): int { return 0; }
    public function advance(): void {}
}
try { (new IteratorIterator(new Failing()))->valid(); }
catch (Exception $error) { echo $error->getMessage() . "\n"; }
--EXPECT--
true
advance:0
valid:1
key:1
value:1
advance:1
valid:2
key:2
value:2
advance:2
valid:3
b:2
a:3
valid:0
key:0
value:0
advance:0
valid:1
key:1
value:1
advance:1
valid:2
key:2
value:2
advance:2
valid:3
a:3
b:2
rewind
key:0
value:0
a:1
delegated
