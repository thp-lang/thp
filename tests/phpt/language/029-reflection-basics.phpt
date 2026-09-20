--TEST--
reflection exposes class, member, function, and type descriptors
--FILE--
<?thp

class Counter {
    private int $value = 4;

    public function __construct(int $value = 4) {
        $this->value = $value;
    }

    private function add(int $amount = 1): int {
        return $this->value + $amount;
    }
}

interface Marker {}

trait Feature {}

class NoDefault {
    public int $value;

    public function __construct(int $value) {
        $this->value = $value;
    }
}

function surround(string $value, string $left = "[", string $right = "]"): string {
    return $left . $value . $right;
}

function nullable(?string $value): ?string {
    return $value;
}

$class = new ReflectionClass("\\Counter");
echo $class->getName() . ":" . $class->getShortName() . "\n";
echo $class->isInstantiable() . ":" . $class->isUserDefined() . "\n";
echo (new ReflectionClass("Marker"))->isInterface() . ":" . (new ReflectionClass("Feature"))->isTrait() . "\n";
$property = $class->getDeclaredProperty("value");
echo $property->getName() . ":" . $property->getType()->getDisplayName() . ":" . $property->isPrivate() . "\n";
var_dump($property->getDefaultValue());
var_dump((new ReflectionProperty("NoDefault", "value"))->getDefaultValue());
$object = $class->newInstanceArgs([8]);
var_dump($property->getValue($object));
$property->setValue($object, 10);
$class->getConstructor()->invokeArgs($object, [11]);
$method = $class->getDeclaredMethod("add");
echo $method->isPrivate() . ":" . $method->getDeclaringClass()->getName() . "\n";
var_dump($method->invokeArgs($object, [2]));
$function = new ReflectionFunction("surround");
var_dump($function->invokeArgs(["x"]));
var_dump($function->invokeArgs({"right" => ">", "value" => "x", "left" => "<"}));
$nullable = new ReflectionFunction("nullable");
$type = $nullable->getParameters()[0]->getType();
echo $type->getDisplayName() . ":" . $type->allowsNull() . ":" . $type->equals($nullable->getReturnType()) . "\n";
--EXPECT--
Counter:Counter
true:true
true:true
value:int:true
int(4)
NULL
int(8)
true:Counter
int(13)
string(3) "[x]"
string(3) "<x>"
null|string:true:true
