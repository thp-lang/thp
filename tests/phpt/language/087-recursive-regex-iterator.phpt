--TEST--
recursive regex iterator searches children of unmatched parents
--FILE--
<?thp
class Leaves implements RecursiveIterator<int, string> {
    public int $position = 0;
    public function rewind(): void { $this->position = 0; }
    public function valid(): bool { return $this->position < 2; }
    public function key(): int { return $this->position; }
    public function value(): RecursiveEntry<int, string> {
        if ($this->position === 0) { return new RecursiveEntry<int, string>('hit-child'); }
        return new RecursiveEntry<int, string>('miss-child');
    }
    public function advance(): void { $this->position = $this->position + 1; }
}
class Root implements RecursiveIterator<int, string> {
    public int $position = 0;
    public function rewind(): void { $this->position = 0; }
    public function valid(): bool { return $this->position < 1; }
    public function key(): int { return $this->position; }
    public function value(): RecursiveEntry<int, string> { return new RecursiveEntry<int, string>('miss-root', new Leaves()); }
    public function advance(): void { $this->position = $this->position + 1; }
}
$iter = new RecursiveRegexIterator(new Root(), '/^hit/');
foreach (new RecursiveIteratorIterator<int, string>($iter) as $value) { echo $value; }
--EXPECT--
hit-child
