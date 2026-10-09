--TEST--
infinite iterator advance crosses replay boundaries without intervening reads
--FILE--
<?thp
class Repeatable implements IteratorAggregate<int, int> {
    public function getIterator(): Traversable<int, int> {
        $values: vector<int> = [4, 5];
        return new VectorIterator($values);
    }
}
$repeat = new InfiniteIterator(new Repeatable());
$repeat->rewind();
$repeat->advance();
$repeat->advance();
$repeat->advance();
echo $repeat->value();
--EXPECT--
5
