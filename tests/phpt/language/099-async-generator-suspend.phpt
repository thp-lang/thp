--TEST--
generator bodies suspend while preserving their iterator state
--FILE--
<?thp
function numbers(): Generator<int, int> {
    echo "A";
    suspend();
    echo "C";
    yield 7;
}
scheduler_register_default();
$reader = async(function (): void {
    $items = numbers();
    $items->rewind();
    echo $items->value();
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABC7
