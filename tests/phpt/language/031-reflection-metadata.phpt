--TEST--
reflection retains generic, trait-origin, parameter, and discovery metadata
--FILE--
<?thp

trait Labels {
    public string $label = "label";

    public function label(): string {
        return $this->label;
    }
}

class Box<T> {
    use Labels;

    public T $item;

    public function __construct(T $item) {
        $this->item = $item;
    }

    public static function choose(T $left, int $right = 3): T {
        return $left;
    }
}

function total(int $left, int $right = 3): int {
    return $left + $right;
}

function variadicTotal(int $base, int ...$values): int {
    foreach ($values as $value) {
        $base = $base + $value;
    }
    return $base;
}

$box = new Box<int>(8);
$class = new ReflectionClass($box);
echo $class->getType()->getDisplayName() . "\n";
echo $class->getType()->getTypeArguments()[0]->getDisplayName() . "\n";
var_dump((new ReflectionProperty($box, "item"))->getValue($box));
$method = new ReflectionMethod($box, "choose");
var_dump($method->invokeArgs($box, [7]));
echo $method->getNumberOfParameters() . ":" . $method->getNumberOfRequiredParameters() . "\n";
echo $method->getParameters()[1]->getName() . "\n";
var_dump($method->getParameters()[1]->getDefaultValue());
echo $class->getDeclaredMethod("label")->getOriginTrait()->getName() . ":";
echo $class->getDeclaredProperty("label")->getOriginTrait()->getName() . "\n";
$function = new ReflectionFunction("\\total");
$functionParameter = new ReflectionParameter("total", "right");
echo $functionParameter->getType()->getDisplayName() . ":" . $functionParameter->getPosition() . "\n";
var_dump($function->invokeArgs({"left" => 7}));
$variadic = new ReflectionFunction("variadicTotal");
$methodParameter = new ReflectionParameter([$box, "choose"], 1);
echo $methodParameter->getName() . "\n";
var_dump($methodParameter->getDefaultValue());
echo $variadic->getParameters()[1]->getType()->getDisplayName() . ":" . $variadic->getParameters()[1]->isVariadic() . "\n";
var_dump($variadic->invokeArgs([1, 2, 3]));
var_dump((new ReflectionClass("Box"))->getType());
--EXPECT--
Box<int>
int
int(8)
int(7)
2:1
right
int(3)
Labels:Labels
int:1
int(10)
right
int(3)
int:true
int(6)
NULL
