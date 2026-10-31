--TEST--
iterator_to_vector yields during file line advancement
--FILE--
<?thp
scheduler_register_default();
$file = new SplFileObject("001-memory-binary-roundtrip.phpt");
$file->rewind();
$reader = async(function () use ($file): void {
    echo "A";
    $lines: vector<string> = iterator_to_vector($file);
    if (count($lines) > 0) { echo "C"; }
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABC
