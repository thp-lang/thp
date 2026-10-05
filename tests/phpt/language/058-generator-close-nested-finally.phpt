--TEST--
closing inside nested finally runs each active cleanup once
--FILE--
<?thp
function values(): Generator<int, int> {
    try {
        try { return 5; }
        finally { echo "inner\n"; yield 1; }
    } finally { echo "outer\n"; }
}
$generator = values();
$generator->rewind();
echo $generator->value() . "\n";
$generator->close();
echo "closed\n";
--EXPECT--
inner
1
outer
closed
