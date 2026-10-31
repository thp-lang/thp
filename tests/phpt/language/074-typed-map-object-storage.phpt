--TEST--
typed map preserves key order and object storage keys by identity
--FILE--
<?thp

$map = new TypedMap<string, int>();
$map->set("first", 1);
$map->set("second", 2);
$map->set("first", 3);
echo $map->count() . ":" . ($map->get("first") === 3) . "\n";
foreach ($map as $key => $value) { echo $key . ":" . $value . "\n"; }
echo $map->contains("missing");
echo "\n";
echo $map->remove("first");
echo ":";
echo $map->remove("first");
echo "\n";
echo $map->toMap()["second"] . "\n";

class Box {
    public int $value = 7;
}
$a = new Box();
$b = new Box();
$storage = new ObjectStorage<int>();
$storage->attach($a, 10);
$storage->attach($b, 20);
$storage->attach($a, 11);
echo $storage->count() . ":" . ($storage->get($a) === 11) . ":" . ($storage->get($b) === 20) . "\n";
foreach ($storage as $object) { echo $object === $a; echo "\n"; }
echo $storage->detach($a);
echo ":";
echo $storage->contains($a);
echo "\n";
try { $storage->attach(1, 30); }
catch (TypeError $error) { echo "object required\n"; }
--EXPECT--
2:true
first:3
second:2
false
true:false
2
2:true:true
true
false
true:false
object required
