--TEST--
caching iterator supports user subclasses
--FILE--
<?thp
class MyCache extends CachingIterator<int, int> {
    public int $marker = 7;
}
$values: vector<int> = [1];
$cache = new MyCache($values);
foreach ($cache as $value) { echo $value; }
echo ":" . $cache->marker;
--EXPECT--
1:7
