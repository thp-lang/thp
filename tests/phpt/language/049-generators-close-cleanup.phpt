--TEST--
closing a suspended generator runs using and finally once
--FILE--
<?thp
class Resource implements Closeable {
    public function close(): void { echo "resource closed\n"; }
    public function isClosed(): bool { return false; }
}
function values(): Generator<int, int> {
    try {
        using ($resource = new Resource()) {
            echo "entered\n";
            yield 1;
            echo "resumed\n";
        }
    } catch (Throwable $error) {
        echo "caught close\n";
    } finally {
        echo "finally\n";
    }
}
$g = values();
$g->rewind();
$g->close();
$g->close();
echo $g->valid() . "\n";
--EXPECT--
entered
resource closed
finally
false
