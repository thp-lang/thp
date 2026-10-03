--TEST--
raw vectors do not expose iterator cursor methods
--FILE--
<?thp

$values: vector<int> = [1];
$values->rewind();
--EXPECTF--
%s041-raw-collection-cursor.phpt:4:1: error[T0402]: method calls require an object, found `vector<int>`
    4 | $values->rewind();
      | ^^^^^^^
