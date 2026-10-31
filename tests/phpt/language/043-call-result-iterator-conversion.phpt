--TEST--
collection returned by a call converts at a typed iterator boundary
--FILE--
<?thp

function values(): vector<int> { return [7, 8]; }

$cursor: Iterator<int, int> = values();
echo $cursor->value() . "\n";
echo iterator_count($cursor) . "\n";
--EXPECT--
7
2
