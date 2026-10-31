--TEST--
generator key overflow runs active finally cleanup
--FILE--
<?thp
function keys(): Generator<int, int> {
    try {
        yield 9223372036854775807 => 1;
        yield 2;
    } finally {
        echo "cleanup\n";
    }
}
$g = keys();
$g->rewind();
try { $g->advance(); } catch (ValueError $error) { echo "overflow\n"; }
--EXPECT--
cleanup
overflow
