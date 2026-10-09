--TEST--
typed map get returns optional union values
--FILE--
<?thp
$map = new TypedMap<string, int|string>();
$map->set("a", 7);
echo $map->get("a") !== null;
--EXPECT--
true
