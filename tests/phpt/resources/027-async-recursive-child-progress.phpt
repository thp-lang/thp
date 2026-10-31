--TEST--
recursive directory child checks yield to ready coroutines
--FILE--
<?thp
scheduler_register_default();
$tree = new RecursiveDirectoryIterator("..");
$reader = async(function () use ($tree): void {
    echo "A";
    $entry = $tree->value();
    echo "C";
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABC
