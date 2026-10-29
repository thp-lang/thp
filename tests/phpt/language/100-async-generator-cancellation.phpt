--TEST--
cancellation reaches a suspended generator and its catch may return a value
--FILE--
<?thp
function guarded(): Generator<int, int> {
    try {
        echo "A";
        suspend();
        yield 1;
    } catch (CancellationException $error) {
        echo "C";
        yield 2;
    }
}
scheduler_register_default();
$reader = async(function (): void {
    $items = guarded();
    $items->rewind();
    echo $items->value();
});
$canceller = async(function () use ($reader): void { echo "B"; $reader->cancel(); });
await($reader);
await($canceller);
--EXPECT--
ABC2
