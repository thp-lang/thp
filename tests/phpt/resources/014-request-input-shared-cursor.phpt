--TEST--
thp:/input exposes one shared binary request cursor
--FILE--
<?thp
$first = Streams::open("thp:/input", OpenMode::Read);
$second = Streams::open("thp:/input", OpenMode::Read);
echo $first->read(3);
echo $second->read(3);
var_dump($first === $second);
--STDIN--
abcdef
--EXPECT--
abcdefbool(true)
