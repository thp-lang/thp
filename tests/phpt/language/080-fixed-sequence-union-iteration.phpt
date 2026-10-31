--TEST--
fixed sequence iterates nullable slots with a union element type
--FILE--
<?thp
$values = new FixedSequence<int|string>(2);
$values->offsetSet(0, 7);
$values->offsetSet(1, "x");
foreach ($values as $value) { echo $value !== null; }
--EXPECT--
truetrue
