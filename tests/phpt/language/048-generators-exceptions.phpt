--TEST--
generator failures propagate on the resume that executes the throw
--FILE--
<?thp
function before(): Generator<int, int> {
    throw new Exception("before");
    yield 1;
}
function after(): Generator<int, int> {
    yield 1;
    throw new Exception("after");
}
$first = before();
echo "lazy\n";
try { $first->rewind(); } catch (Exception $error) { echo $error->getMessage() . "\n"; }
echo $first->valid() . "\n";
$second = after();
$second->rewind();
echo $second->value() . "\n";
try { $second->advance(); } catch (Exception $error) { echo $error->getMessage() . "\n"; }
echo $second->valid() . "\n";
--EXPECT--
lazy
before
false
1
after
false
