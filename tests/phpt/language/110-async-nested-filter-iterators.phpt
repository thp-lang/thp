--TEST--
nested filter iterators preserve their scan state across suspension
--FILE--
<?thp
class SlowEvenFilter extends FilterIterator<int, int> {
    public function accept(int $value, int $key): bool {
        echo "A";
        suspend();
        return $value % 2 == 0;
    }
}
scheduler_register_default();
$reader = async(function (): void {
    $outer = new CallbackFilterIterator(
        new SlowEvenFilter([1, 2]),
        fn(int $value, int $key): bool => true,
    );
    $outer->rewind();
    if ($outer->valid()) { echo $outer->value(); }
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABA2
