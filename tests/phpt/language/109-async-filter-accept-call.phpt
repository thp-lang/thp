--TEST--
direct callback filter acceptance suspends through a coroutine frame
--FILE--
<?thp
scheduler_register_default();
$filter = new CallbackFilterIterator([1], function (int $value, int $key): bool {
    echo "A";
    suspend();
    echo "C";
    return true;
});
$reader = async(function () use ($filter): void {
    if ($filter->accept(1, 0)) { echo "D"; }
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABCD
