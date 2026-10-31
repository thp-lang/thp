--TEST--
reflection rejects incompatible receivers, values, arguments, and construction
--FILE--
<?thp

class Item {
    public int $value = 1;

    public function read(): int {
        return $this->value;
    }
}

class Other {}

interface ItemInterface {}

trait ItemTrait {}

class CustomReflectionException extends ReflectionException {}

class GenericBox<T> {
    public T $value;

    public function __construct(T $value) {
        $this->value = $value;
    }
}

abstract class AbstractItem {}

class PrivateItem {
    private function __construct() {}
}

function combine(int $left, int $right): int {
    return $left + $right;
}

$property = (new ReflectionClass("Item"))->getProperty("value");
$itemClass = new ReflectionClass("Item");
var_dump($itemClass->getDeclaredMethod("missing"));
var_dump($itemClass->getDeclaredProperty("missing"));
try {
    $itemClass->getMethod("missing");
} catch (ReflectionException $error) {
    echo "get-method\n";
}
try {
    $itemClass->getProperty("missing");
} catch (ReflectionException $error) {
    echo "get-property\n";
}
try {
    $property->setValue(new Item(), "wrong");
} catch (ReflectionException $error) {
    echo "property-type\n";
}
try {
    $property->getValue(new Other());
} catch (ReflectionException $error) {
    echo "property-receiver\n";
}
try {
    (new ReflectionProperty(new GenericBox<int>(1), "value"))->getValue(new GenericBox<string>("wrong"));
} catch (ReflectionException $error) {
    echo "generic-receiver\n";
}
try {
    (new ReflectionMethod("Item", "read"))->invokeArgs(null);
} catch (ReflectionException $error) {
    echo "method-receiver\n";
}
try {
    (new ReflectionFunction("combine"))->invokeArgs({"unknown" => 1});
} catch (Error $error) {
    echo "named-argument\n";
}
try {
    (new ReflectionClass("AbstractItem"))->newInstanceArgs();
} catch (Error $error) {
    echo "abstract\n";
}
try {
    (new ReflectionClass("ItemInterface"))->newInstanceArgs();
} catch (Error $error) {
    echo "interface\n";
}
try {
    (new ReflectionClass("ItemTrait"))->newInstanceArgs();
} catch (Error $error) {
    echo "trait\n";
}
try {
    (new ReflectionClass("PrivateItem"))->newInstanceArgs();
} catch (ReflectionException $error) {
    echo "private-constructor\n";
}
try {
    (new ReflectionClass("Item"))->newInstanceArgs();
} catch (ReflectionException $error) {
    echo "missing-constructor\n";
}
try {
    (new ReflectionClass("GenericBox"))->newInstanceArgs([1]);
} catch (Error $error) {
    echo "incomplete-generic\n";
}
try {
    (new ReflectionFunction("combine"))->invokeArgs([1]);
} catch (TypeError $error) {
    echo "argument-count\n";
}
try {
    (new ReflectionFunction("combine"))->invokeArgs(["wrong", 2]);
} catch (TypeError $error) {
    echo "argument-type\n";
}
try {
    new ReflectionClass(1);
} catch (TypeError $error) {
    echo "class-type\n";
}
try {
    new ReflectionClass("Missing");
} catch (ReflectionException $error) {
    echo "missing-class\n";
}
try {
    new ReflectionFunction("Missing");
} catch (ReflectionException $error) {
    echo "missing-function\n";
}
try {
    new ReflectionMethod("Item", "missing");
} catch (ReflectionException $error) {
    echo "missing-method\n";
}
try {
    new ReflectionProperty("Item", "missing");
} catch (ReflectionException $error) {
    echo "missing-property\n";
}
try {
    new ReflectionParameter("combine", "missing");
} catch (ReflectionException $error) {
    echo "missing-parameter\n";
}
try {
    new ReflectionParameter(["Item"], 0);
} catch (TypeError $error) {
    echo "callable-shape\n";
}
try {
    new ReflectionClass("\xff");
} catch (ReflectionException $error) {
    echo "invalid-name\n";
}
--EXPECT--
NULL
NULL
get-method
get-property
property-type
property-receiver
generic-receiver
method-receiver
named-argument
abstract
interface
trait
private-constructor
missing-constructor
incomplete-generic
argument-count
argument-type
class-type
missing-class
missing-function
missing-method
missing-property
missing-parameter
callable-shape
invalid-name
