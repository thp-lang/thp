--TEST--
reflection invocation of a file method yields while reading
--FILE--
<?thp
scheduler_register_default();
$file = new SplFileObject("001-memory-binary-roundtrip.phpt");
$method = new ReflectionMethod("SplFileObject", "fread");
$reader = async(function () use ($file, $method): void {
    echo "A";
    $result = $method->invokeArgs($file, [7]);
    if (is_string($result)) { echo $result; }
    echo "C";
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
AB--TEST-C
