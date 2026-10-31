--TEST--
filter iterators preserve keys and iterator_apply consumes through its false callback
--FILE--
<?thp

class EvenFilter extends FilterIterator<int, int> {
    public function accept(int $value, int $key): bool { return $value % 2 == 0; }
}

$values: vector<int> = [1, 2, 3, 4];
$custom = new EvenFilter($values);
foreach ($custom as $key => $value) { echo "c" . $key . ":" . $value . "\n"; }
$callback = new CallbackFilterIterator($values, fn(int $value, int $key): bool => $key > 1);
foreach ($callback as $key => $value) { echo "b" . $key . ":" . $value . "\n"; }
$cursor: Iterator<int, int> = $values;
echo iterator_apply($cursor, fn(int $value, int $key): bool => $key < 1) . "\n";
echo $cursor->value() . "\n";
$empty: vector<int> = [];
echo iterator_count(new EvenFilter($empty)) . "\n";
echo iterator_apply($empty, fn(int $value, int $key): bool => true) . "\n";
--EXPECT--
c1:2
c3:4
b2:3
b3:4
2
3
0
0
