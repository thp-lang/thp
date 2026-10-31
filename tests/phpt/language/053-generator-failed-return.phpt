--TEST--
uncaught generator exception has no final return value
--FILE--
<?thp
function broken(): Generator<int, int> {
    yield 1;
    throw new Exception("boom");
}
$g = broken();
$g->rewind();
try { $g->advance(); } catch (Exception $error) { echo "caught\n"; }
try { var_dump($g->getReturn()); } catch (Error $error) { echo "no return\n"; }
--EXPECT--
caught
no return
