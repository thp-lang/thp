--TEST--
directory and glob construction yield to ready coroutines
--FILE--
<?thp
scheduler_register_default();
$directory = async(function (): void {
    $items = new DirectoryIterator(".");
    if ($items->isDot()) { echo "C"; }
});
$other = async(function (): void { echo "B"; });
echo "A";
await($directory);
await($other);
$glob = async(function (): void {
    $items = new GlobIterator("*.phpt");
    if ($items->count() > 0) { echo "F"; }
});
$other = async(function (): void { echo "E"; });
echo "D";
await($glob);
await($other);
--EXPECT--
ABCDEF
