--TEST--
failed yield during generator close still closes outer using resource
--FILE--
<?thp
class Resource implements Closeable {
    public function close(): void { echo "closed\n"; }
    public function isClosed(): bool { return false; }
}
function values(): Generator<int, int> {
    using ($resource = new Resource()) {
        try { yield 1; }
        finally { yield 2; }
    }
}
$g = values();
$g->rewind();
try { $g->close(); } catch (Error $error) { echo "yield blocked\n"; }
--EXPECT--
closed
yield blocked
