--TEST--
collection conversion at typed boundaries and direct foreach traversal agree
--FILE--
<?thp

function countRemaining(Iterator<int, int> $cursor): int { return iterator_count($cursor); }
function make(): Traversable<int, int> { return [3, 4]; }

class Holder {
    public Iterator<int, int> $cursor;
    public function __construct(Iterator<int, int> $cursor) { $this->cursor = $cursor; }
}

$values: vector<int> = [3, 4];
echo countRemaining($values) . ":" . countRemaining($values) . "\n";
$holder = new Holder($values);
echo $holder->cursor->value() . "\n";
$holder->cursor = [8, 9];
echo $holder->cursor->value() . "\n";
$nullable: Iterator<int, int>|null = [];
echo ($nullable === null) . "\n";
$emptyMap: Iterator<string, int> = {};
echo iterator_count($emptyMap) . "\n";
$emptyVector = new VectorIterator<int>([]);
echo $emptyVector->valid() . "\n";
foreach (make() as $key => $value) { echo $key . ":" . $value . "\n"; }

$direct: vector<int> = [1, 2, 3];
foreach ($direct as $key => $value) {
    $direct[0] = 99;
    if ($key === 0) { continue; }
    echo "d" . $key . ":" . $value . "\n";
    break;
}
$wrapped = new IteratorIterator(new VectorIterator([1, 2, 3]));
foreach ($wrapped as $key => $value) {
    if ($key === 0) { continue; }
    echo "w" . $key . ":" . $value . "\n";
    break;
}
$wrappedCollection = new IteratorIterator([6, 7]);
echo iterator_count($wrappedCollection) . "\n";

$map: map<string, int> = {"a" => 1, "b" => 2};
$cursor = new MapIterator($map);
$map["a"] = 9;
foreach ($cursor as $key => $value) { echo $key . ":" . $value . "\n"; }
--EXPECT--
2:2
3
8
false
0
false
0:3
1:4
d1:2
w1:2
2
a:1
b:2
