--TEST--
recursive iterator iterator validates modes and depths and handles empty cursors
--FILE--
<?thp

class EmptyTree implements RecursiveIterator<int, int> {
    public function rewind(): void {}
    public function valid(): bool { return false; }
    public function key(): int { return 0; }
    public function value(): RecursiveEntry<int, int> { return new RecursiveEntry<int, int>(0); }
    public function advance(): void {}
}

try { new RecursiveIteratorIterator<int, int>(new EmptyTree(), 3); }
catch (ValueError $error) { echo "mode\n"; }
$flat = new RecursiveIteratorIterator<int, int>(new EmptyTree());
try { $flat->setMaxDepth(-2); }
catch (ValueError $error) { echo "depth\n"; }
echo $flat->getMaxDepth() . "\n";
echo $flat->getSubIterator() === null;
echo "\n";
foreach ($flat as $value) { echo $value; }
echo $flat->valid();
echo "\n";
echo $flat->getSubIterator(-1) === null;
echo "\n";
foreach ($flat as $value) { echo $value; }
echo "done\n";
--EXPECT--
mode
depth
false
true
false
true
done
