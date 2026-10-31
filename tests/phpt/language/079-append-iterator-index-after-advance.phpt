--TEST--
append iterator index follows transitions and exhaustion immediately
--FILE--
<?thp
$first: vector<int> = [1];
$second: vector<int> = [2];
$all = new AppendIterator<int, int>();
$all->append($first);
$all->append($second);
$all->rewind();
$all->advance();
echo ($all->getIteratorIndex() === 1) . ":" . $all->value() . "\n";
$all->advance();
echo $all->getIteratorIndex() === null;
--EXPECT--
true:2
true
