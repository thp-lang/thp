--TEST--
recursive walker yields while checking file-backed children
--FILE--
<?thp
scheduler_register_default();
$tree = new RecursiveDirectoryIterator(".");
$walk = new RecursiveIteratorIterator<string, SplFileInfo>($tree);
$reader = async(function () use ($walk): void {
    echo "A";
    $walk->rewind();
    if ($walk->valid()) { echo "C"; }
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABC
