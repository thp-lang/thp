--TEST--
direct collection and wrapper foreach agree on keys, values, transfers, and mutation snapshots
--FILE--
<?thp

$directMap: map<string, int> = {"a" => 1, "b" => 2, "c" => 3};
foreach ($directMap as $key => $value) {
    $directMap["a"] = 9;
    if ($key === "a") { continue; }
    echo "direct:" . $key . ":" . $value . "\n";
    break;
}
$wrappedMap: map<string, int> = {"a" => 1, "b" => 2, "c" => 3};
$wrapper = new IteratorIterator(new MapIterator($wrappedMap));
foreach ($wrapper as $key => $value) {
    $wrappedMap["a"] = 9;
    if ($key === "a") { continue; }
    echo "wrapped:" . $key . ":" . $value . "\n";
    break;
}

$directVector: vector<int> = [1, 2, 3];
foreach ($directVector as $value) {
    $directVector[0] = 9;
    if ($value === 1) { continue; }
    echo "direct-value:" . $value . "\n";
    break;
}
$wrappedVector: vector<int> = [1, 2, 3];
$vectorWrapper = new IteratorIterator(new VectorIterator($wrappedVector));
foreach ($vectorWrapper as $value) {
    $wrappedVector[0] = 9;
    if ($value === 1) { continue; }
    echo "wrapped-value:" . $value . "\n";
    break;
}
--EXPECT--
direct:b:2
wrapped:b:2
direct-value:2
wrapped-value:2
