--TEST--
recursive callback filter suspends and preserves its child wrapper
--FILE--
<?thp
class OneLeaf implements RecursiveIterator<int, int> {
    public int $position = 0;
    public function rewind(): void { $this->position = 0; }
    public function valid(): bool { return $this->position < 1; }
    public function key(): int { return 0; }
    public function value(): RecursiveEntry<int, int> { return new RecursiveEntry<int, int>(2); }
    public function advance(): void { $this->position = $this->position + 1; }
}
class OneParent implements RecursiveIterator<int, int> {
    public int $position = 0;
    public function rewind(): void { $this->position = 0; }
    public function valid(): bool { return $this->position < 1; }
    public function key(): int { return 0; }
    public function value(): RecursiveEntry<int, int> {
        return new RecursiveEntry<int, int>(8, new OneLeaf());
    }
    public function advance(): void { $this->position = $this->position + 1; }
}
scheduler_register_default();
$filter = new RecursiveCallbackFilterIterator<int, int>(
    new OneParent(),
    function (RecursiveEntry<int, int> $entry, int $key): bool {
        echo "A";
        suspend();
        echo "C";
        return true;
    },
);
$reader = async(function () use ($filter): void {
    $filter->rewind();
    if ($filter->valid()) {
        $entry = $filter->value();
        echo $entry->value();
        if ($entry->children() === null) { echo "N"; } else { echo "Y"; }
    }
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABC8Y
