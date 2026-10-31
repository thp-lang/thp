--TEST--
caching iterator provides one-entry lookahead and a keyed cache
--FILE--
<?thp

$source: map<string, int> = {"a" => 1, "b" => 2};
$cache = new CachingIterator($source, CachingIterator::FULL_CACHE);
foreach ($cache as $key => $value) {
    echo $key . ":" . $value . ":" . $cache->hasNext() . ":" . $cache->__toString() . "\n";
}
echo count($cache) . "\n";
echo $cache->offsetExists("a") . ":" . $cache->offsetGet("b") . "\n";
$snapshot = $cache->getCache();
foreach ($snapshot as $key => $value) { echo $key . "=" . $value . "\n"; }
$cache->offsetUnset("a");
echo $cache->offsetExists("a") . "\n";
$cache->offsetSet("c", 3);
echo $cache->offsetGet("c") . "\n";
--EXPECT--
a:1:true:1
b:2:false:2
2
true:2
a=1
b=2
false
3
