--TEST--
reflection retains exact parent and child private property and method declarations
--FILE--
<?thp

class ParentValue {
    private int $value = 1;

    private function read(): int {
        return $this->value;
    }
}

class ChildValue extends ParentValue {
    private string $value = "child";

    private function read(): string {
        return $this->value;
    }
}

$object = new ChildValue();
$parent = new ReflectionClass("ParentValue");
$child = new ReflectionClass($object);
$parentProperty = $parent->getDeclaredProperty("value");
$childProperty = $child->getDeclaredProperty("value");
var_dump($parentProperty->getValue($object));
var_dump($childProperty->getValue($object));
$parentProperty->setValue($object, 7);
$childProperty->setValue($object, "updated");
var_dump($parent->getDeclaredMethod("read")->invokeArgs($object));
var_dump($child->getDeclaredMethod("read")->invokeArgs($object));
echo $child->getProperties()[0]->getName() . ":" . $child->getMethods()[0]->getName() . "\n";
--EXPECT--
int(1)
string(5) "child"
int(7)
string(7) "updated"
value:read
