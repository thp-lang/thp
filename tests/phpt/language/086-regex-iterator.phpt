--TEST--
regex iterator filters and transforms values with constant modes
--FILE--
<?thp
$values = ['foo2', 'bar', 'zap3'];
$matched = new RegexIterator(new VectorIterator($values), '/([a-z]+)([0-9]+)/', RegexIterator::GET_MATCH);
foreach ($matched as $capture) { echo $capture['1']; }
$replaced = new RegexIterator(new VectorIterator($values), '/[0-9]+/', RegexIterator::REPLACE, false, 0, '!');
foreach ($replaced as $value) { echo $value; }
$inverted = new RegexIterator(new VectorIterator($values), '/[0-9]+/', RegexIterator::MATCH, true);
foreach ($inverted as $value) { echo $value; }
--EXPECT--
foozapfoo!zap!bar
