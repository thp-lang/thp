--TEST--
recursive walker hooks suspend without losing traversal state
--FILE--
<?thp
class HookLeaf implements RecursiveIterator<int, int> {
    public int $position = 0;
    public function rewind(): void { $this->position = 0; }
    public function valid(): bool { return $this->position < 1; }
    public function key(): int { return 0; }
    public function value(): RecursiveEntry<int, int> { return new RecursiveEntry<int, int>(2); }
    public function advance(): void { $this->position = $this->position + 1; }
}
class SlowWalk extends RecursiveIteratorIterator<int, int> {
    public function beginIteration(): void { echo "A"; suspend(); echo "C"; }
    public function nextElement(): void { echo "D"; }
    public function endIteration(): void { echo "E"; }
}
scheduler_register_default();
$reader = async(function (): void {
    $walk = new SlowWalk(new HookLeaf());
    foreach ($walk as $value) { echo $value; }
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABCD2E
