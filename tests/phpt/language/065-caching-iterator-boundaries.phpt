--TEST--
caching iterator validates flags and keeps empty cursors exhausted
--FILE--
<?thp

$empty: vector<int> = [];
$cache = new CachingIterator($empty);
echo iterator_count($cache) . ":" . $cache->hasNext() . ":" . count($cache) . "\n";
try { $cache->getCache(); }
catch (LogicException $error) { echo "disabled\n"; }
try { $cache->setFlags(12345); }
catch (ValueError $error) { echo "flags\n"; }
$full = new CachingIterator($empty, CachingIterator::FULL_CACHE);
try { $full->offsetSet(null, 1); }
catch (ValueError $error) { echo "key\n"; }
--EXPECT--
0:false:0
disabled
flags
key
