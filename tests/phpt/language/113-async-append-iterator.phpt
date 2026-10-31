--TEST--
append iterator resumes when entering a suspended inner iterator
--FILE--
<?thp
class SlowAppendSource implements Iterator<int, int> {
    public function rewind(): void { echo "A"; suspend(); }
    public function valid(): bool { return true; }
    public function key(): int { return 0; }
    public function value(): int { return 7; }
    public function advance(): void {}
}
scheduler_register_default();
$reader = async(function (): void {
    $all = new AppendIterator<int, int>();
    $empty: vector<int> = [];
    $all->append($empty);
    $all->append(new SlowAppendSource());
    $all->rewind();
    if ($all->valid()) { echo $all->value(); }
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
AB7
