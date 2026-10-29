--TEST--
iterator conversion suspends through delegated user iterator methods
--FILE--
<?thp
class SlowIterator implements Iterator<int, int> {
    public int $position = 0;
    public function rewind(): void { $this->position = 0; }
    public function valid(): bool {
        echo "A";
        suspend();
        return $this->position < 1;
    }
    public function key(): int { return $this->position; }
    public function value(): int { return 7; }
    public function advance(): void { $this->position = $this->position + 1; }
}
scheduler_register_default();
$reader = async(function (): void {
    $values: vector<int> = iterator_to_vector(new IteratorIterator(new SlowIterator()));
    echo $values[0];
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABA7
