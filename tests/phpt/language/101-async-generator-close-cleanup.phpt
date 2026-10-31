--TEST--
closing a generator can suspend inside finally without losing cleanup state
--FILE--
<?thp
function cleanup(): Generator<int, int> {
    try { yield 1; }
    finally { echo "A"; suspend(); echo "C"; }
}
scheduler_register_default();
$items = cleanup();
$items->rewind();
$closer = async(function () use ($items): void { $items->close(); });
$other = async(function (): void { echo "B"; });
await($closer);
await($other);
--EXPECT--
ABC
