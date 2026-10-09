--TEST--
iterator adapters reject invalid positions and terminate empty replay
--FILE--
<?thp

$values: vector<int> = [1, 2];
try { new LimitIterator($values, -1); }
catch (ValueError $error) { echo "offset\n"; }
try { new LimitIterator($values, 0, -2); }
catch (ValueError $error) { echo "limit\n"; }
$window = new LimitIterator($values, 1, 1);
try { $window->seek(1); }
catch (OutOfBoundsException $error) { echo "seek\n"; }

class EmptySource implements IteratorAggregate<int, int> {
    public function getIterator(): Traversable<int, int> {
        $empty: vector<int> = [];
        return new VectorIterator($empty);
    }
}
echo iterator_count(new InfiniteIterator(new EmptySource())) . "\n";

$append = new AppendIterator<int, int>();
$append->append($values);
$append->rewind();
try { $append->append($values); }
catch (LogicException $error) { echo "append\n"; }
--EXPECT--
offset
limit
seek
0
append
