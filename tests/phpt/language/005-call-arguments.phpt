--TEST--
default, named, and variadic arguments bind for functions, methods, and constructors
--FILE--
<?thp

function ordered(string $first, string $second = "B"): string {
    return $first . $second;
}

function total(int $base = 10, int ...$values): int {
    foreach ($values as $value) {
        $base = $base + $value;
    }
    return $base;
}

class Label {
    private string $text;

    public function __construct(string $prefix = "item", int $number = 1) {
        $this->text = $prefix . ":" . $number;
    }

    public function decorate(string $left = "[", string $right = "]"): string {
        return $left . $this->text . $right;
    }

    public static function wrap(string $left = "<", string $right = ">"): string {
        return $left . "static" . $right;
    }
}

echo ordered(second: "2", first: "1") . "\n";
echo total(1, 2, 3, 4) . "\n";
$label = new Label(number: 7);
echo $label->decorate(right: "}", left: "{") . "\n";
echo Label::wrap(right: ")", left: "(") . "\n";
echo MemoryStream::open(initial: "native")->readAll() . "\n";
--EXPECT--
12
10
{item:7}
(static)
native
