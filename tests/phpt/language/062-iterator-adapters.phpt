--TEST--
limit, infinite, and append iterators use the common cursor protocol
--FILE--
<?thp

$values: vector<int> = [10, 20, 30];
$limited = new LimitIterator($values, 1, 2);
foreach ($limited as $key => $value) { echo $key . ":" . $value . "\n"; }
$limited->seek(0);
echo $limited->getPosition() . ":" . $limited->value() . "\n";

class Repeatable implements IteratorAggregate<int, int> {
    public function getIterator(): Traversable<int, int> {
        $values: vector<int> = [4, 5];
        return new VectorIterator($values);
    }
}
$infinite = new InfiniteIterator(new Repeatable());
$firstFive = new LimitIterator($infinite, 0, 5);
foreach ($firstFive as $value) { echo $value . "\n"; }

$all = new AppendIterator<int, int>();
$all->append($values);
$more: vector<int> = [40];
$all->append($more);
foreach ($all as $value) { echo $value . "\n"; }
if ($all->getIteratorIndex() === null) { echo "done\n"; }
echo iterator_count($all->getArrayIterator()) . "\n";
--EXPECT--
1:20
2:30
0:20
4
5
4
5
4
10
20
30
40
done
2
