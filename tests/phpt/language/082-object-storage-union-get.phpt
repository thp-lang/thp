--TEST--
object storage get returns optional union data
--FILE--
<?thp
class Box {}
$box = new Box();
$storage = new ObjectStorage<int|string>();
$storage->attach($box, 7);
echo $storage->get($box) !== null;
--EXPECT--
true
