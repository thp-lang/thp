--TEST--
IteratorIterator delegates generator suspension through task frames
--FILE--
<?thp
function wrapped(): Generator<int, int> {
    echo "A";
    suspend();
    echo "C";
    yield 7;
}
scheduler_register_default();
$reader = async(function (): void {
    $outer = new IteratorIterator(new IteratorIterator(wrapped()));
    $outer->rewind();
    echo $outer->value();
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABC7
