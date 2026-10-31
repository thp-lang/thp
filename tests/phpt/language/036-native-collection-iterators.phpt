--TEST--
native collection iterators snapshot, rewind and consume from their current cursor
--FILE--
<?thp

$source: vector<int> = [10, 20, 30];
$iterator: Iterator<int, int> = $source;
$source[0] = 99;
echo $iterator->value() . "\n";
$iterator->advance();
echo iterator_count($iterator) . "\n";
echo $iterator->valid() . "\n";
$iterator->rewind();
foreach (new IteratorIterator($iterator) as $key => $value) {
    echo $key . ":" . $value . "\n";
}

$map: map<string, int> = {"a" => 1, "b" => 2};
$cursor: Iterator<string, int> = $map;
$map["a"] = 9;
$cursor->advance();
$rest: map<string, int> = iterator_to_map($cursor);
foreach ($rest as $key => $value) { echo $key . ":" . $value . "\n"; }

$empty = new EmptyIterator<int, string>();
echo $empty->valid() . "\n";
$vector = iterator_to_vector(new VectorIterator([4, 5]));
foreach ($vector as $value) { echo $value . "\n"; }
$mapCursor = new MapIterator({"left" => 6, "right" => 7});
$mapCursor->advance();
$remaining = iterator_to_vector($mapCursor);
foreach ($remaining as $key => $value) { echo $key . ":" . $value . "\n"; }
--EXPECT--
10
2
false
0:10
1:20
2:30
b:2
false
4
5
0:7
