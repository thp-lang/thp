--TEST--
type guards preserve every compatible member of an existing union
--FILE--
<?thp

$number: int|string|bool = 1;
if (is_numeric($number)) {
    var_dump($number);
}

$vector: vector<int>|string = [7];
if (is_vector($vector)) {
    echo $vector[0] . "\n";
}

$map: map<string, int>|string = {"answer" => 42};
if (is_map($map)) {
    echo $map["answer"] . "\n";
}

class Base {}
class First extends Base {}
class Second extends Base {}

$object: First|Second|string = new First();
if ($object instanceof Base) {
    var_dump($object instanceof First);
}
--EXPECT--
int(1)
7
42
bool(true)
