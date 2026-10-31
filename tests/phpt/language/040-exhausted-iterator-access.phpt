--TEST--
exhausted native iterator key access fails
--FILE--
<?thp

$cursor = new EmptyIterator<int, string>();
echo $cursor->key();
--EXPECTF--
%s040-exhausted-iterator-access.phpt:4:6: runtime error: iterator is exhausted
