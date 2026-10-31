--TEST--
closing a generator suspended in finally does not rerun that finally
--FILE--
<?thp
function values(): Generator<int, int> {
    try { return 5; }
    finally {
        echo "finally\n";
        yield 1;
    }
}
$generator = values();
$generator->rewind();
echo $generator->value() . "\n";
$generator->close();
echo "closed\n";
--EXPECT--
finally
1
closed
