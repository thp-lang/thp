--TEST--
fixed sequence keeps indexed nullable slots and iterates through the aggregate protocol
--FILE--
<?thp

$values = new FixedSequence<int>(2);
echo $values->getSize() . ":" . $values->count() . "\n";
echo $values->offsetExists(0);
echo "\n";
$values->offsetSet(0, 7);
$values->offsetSet(1, 8);
echo $values->offsetGet(0) === 7;
echo ":";
echo $values->offsetGet(1) === 8;
echo "\n";
foreach ($values as $key => $value) { echo $key . ":" . ($value !== null) . "\n"; }
$values->offsetUnset(0);
echo $values->offsetExists(0);
echo "\n";
$values->setSize(3);
echo $values->offsetGet(2) === null;
echo "\n";
$values->setSize(1);
echo $values->getSize() . "\n";
try { $values->offsetGet(1); }
catch (OutOfBoundsException $error) { echo "bounds\n"; }
try { $values->setSize(-1); }
catch (ValueError $error) { echo "size\n"; }
try { new FixedSequence<int>(-1); }
catch (ValueError $error) { echo "construct\n"; }
try { $values->offsetSet(null, 4); }
catch (ValueError $error) { echo "append\n"; }
--EXPECT--
2:2
false
true:true
0:true
1:true
false
true
1
bounds
size
construct
append
