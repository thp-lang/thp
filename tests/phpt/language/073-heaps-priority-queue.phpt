--TEST--
max and min heaps and priority queue extract in order and iterate snapshots
--FILE--
<?thp

$max = new MaxHeap<int>();
$max->insert(3);
$max->insert(8);
$max->insert(5);
echo $max->count() . ":" . $max->top() . "\n";
foreach ($max as $value) { echo "max:" . $value . "\n"; }
echo $max->extract() . ":" . $max->extract() . ":" . $max->extract() . "\n";
try { $max->extract(); }
catch (UnderflowException $error) { echo "max empty\n"; }

$min = new MinHeap<int>();
$min->insert(3);
$min->insert(8);
$min->insert(5);
foreach ($min as $value) { echo "min:" . $value . "\n"; }
echo $min->extract() . "\n";

$priority = new PriorityQueue<string>();
$priority->insert("low", 1);
$priority->insert("first", 9);
$priority->insert("second", 9);
foreach ($priority as $value) { echo "priority:" . $value . "\n"; }
echo $priority->extract() . ":" . $priority->extract() . ":" . $priority->extract() . "\n";
echo $priority->isEmpty();
echo "\n";
try { $priority->extract(); }
catch (UnderflowException $error) { echo "priority empty\n"; }
$floating = new MinHeap<float>();
try { $floating->insert(0.0 / 0.0); }
catch (ValueError $error) { echo "nan\n"; }
--EXPECT--
3:8
max:8
max:5
max:3
8:5:3
max empty
min:3
min:5
min:8
3
priority:first
priority:second
priority:low
first:second:low
true
priority empty
nan
