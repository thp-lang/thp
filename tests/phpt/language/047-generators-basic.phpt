--TEST--
generators are lazy one-shot iterators with keyed yields and a final result
--FILE--
<?thp
function values(): Generator<int, string> {
    echo "start\n";
    yield "first";
    yield 8 => "second";
    yield "third";
    return 42;
}
$g = values();
echo "created\n";
$g->rewind();
$g->rewind();
foreach ($g as $key => $value) { echo $key . ":" . $value . "\n"; }
var_dump($g->getReturn());
$g->close();
var_dump($g->getReturn());
echo $g->valid() . "\n";
try { $g->rewind(); } catch (Error $error) { echo "one-shot\n"; }
--EXPECT--
created
start
0:first
8:second
9:third
int(42)
int(42)
false
one-shot
