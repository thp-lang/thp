--TEST--
generator key overflow, empty return, and cleanup failure are observable on resume or close
--FILE--
<?thp
class Resource implements Closeable {
    public function close(): void { throw new Exception("cleanup"); }
    public function isClosed(): bool { return false; }
}
function keys(): Generator<int, int> {
    yield 9223372036854775807 => 1;
    yield 2;
}
function empty(): Generator<int, int> {
    if (false) { yield 1; }
    return;
}
function resource(): Generator<int, int> {
    using ($resource = new Resource()) { yield 1; }
}
$keys = keys();
$keys->rewind();
echo $keys->key() . "\n";
try { $keys->advance(); } catch (ValueError $error) { echo $error->getMessage() . "\n"; }
echo $keys->valid() . "\n";
$empty = empty();
$empty->rewind();
var_dump($empty->getReturn());
$resource = resource();
$resource->rewind();
try { $resource->close(); } catch (Exception $error) { echo $error->getMessage() . "\n"; }
echo $resource->valid() . "\n";
--EXPECT--
9223372036854775807
generator automatic key overflow
false
NULL
cleanup
false
