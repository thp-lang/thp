--TEST--
linked list queue and stack use aggregate iterators and checked removal
--FILE--
<?thp

$list = new LinkedList<int>();
echo $list->isEmpty();
echo "\n";
$list->push(2);
$list->unshift(1);
echo $list->count() . ":" . $list->bottom() . ":" . $list->top() . "\n";
foreach ($list as $key => $value) { echo $key . ":" . $value . "\n"; }
echo $list->shift() . ":" . $list->pop() . "\n";
try { $list->pop(); }
catch (UnderflowException $error) { echo "empty list\n"; }

$queue = new Queue<int>();
$queue->enqueue(3);
$queue->enqueue(4);
foreach ($queue as $value) { echo "q:" . $value . "\n"; }
echo $queue->dequeue() . ":" . $queue->dequeue() . "\n";

$stack = new Stack<int>();
$stack->push(5);
$stack->push(6);
foreach ($stack as $value) { echo "s:" . $value . "\n"; }
echo $stack->pop() . ":" . $stack->pop() . "\n";
--EXPECT--
true
2:1:2
0:1
1:2
1:2
empty list
q:3
q:4
3:4
s:6
s:5
6:5
