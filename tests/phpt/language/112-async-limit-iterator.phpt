--TEST--
limit iterator rewind and value calls preserve suspended inner methods
--FILE--
<?thp
class SlowWindowSource implements Iterator<int, int> {
    public int $position = 0;
    public function rewind(): void { $this->position = 0; }
    public function valid(): bool {
        echo "A";
        suspend();
        return $this->position < 3;
    }
    public function key(): int { return $this->position; }
    public function value(): int { return $this->position + 1; }
    public function advance(): void { $this->position = $this->position + 1; }
}
scheduler_register_default();
$reader = async(function (): void {
    $window = new LimitIterator(new SlowWindowSource(), 1, 1);
    $window->rewind();
    echo $window->value();
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABA2
