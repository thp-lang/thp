--TEST--
generator functions and closures use the existing Iterator protocol
--FILE--
<?thp
function values(): Iterator<int, int> {
    yield 3;
}
$source = values();
foreach ($source as $key => $value) { echo $key . ":" . $value . "\n"; }
$captured: int = 5;
$make = function () use ($captured): Generator<int, int> {
    yield $captured;
};
$generated = $make();
foreach ($generated as $value) { echo $value . "\n"; }
--EXPECT--
0:3
5
