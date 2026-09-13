--TEST--
dynamic construction uses direct-positive type guards and exact class lookup
--FILE--
<?thp

namespace App;

class Greeter {
    public string $message = "hello";

    public function __construct(string $name = "world") {
        $this->message = $this->message . " " . $name;
    }

    public final function message(): string {
        return $this->message;
    }
}

$class: mixed = "App\\Greeter";
if (is_string($class)) {
    $value = new $class(name: "THP");
    if ($value instanceof Greeter) {
        echo $value->message() . "\n";
    }
}

var_dump(is_numeric(" -1.5e2 "));
var_dump(is_numeric("0x10"));
var_dump(is_int(1));
var_dump(is_float(1.0));
var_dump(is_null(null));
var_dump(is_vector([1]));
var_dump(is_map({"key" => 1}));
try {
    $short = new ("Greeter")();
} catch (\Error $error) {
    echo "exact\n";
}
--EXPECT--
hello THP
bool(true)
bool(false)
bool(true)
bool(true)
bool(true)
bool(true)
bool(true)
exact
