--TEST--
caching iterator prefetch yields while reading a file-backed cursor
--FILE--
<?thp
scheduler_register_default();
$file = new SplFileObject("001-memory-binary-roundtrip.phpt");
$cache = new CachingIterator($file);
$reader = async(function () use ($cache): void {
    echo "A";
    $cache->rewind();
    if ($cache->valid()) { echo "C"; }
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABC
