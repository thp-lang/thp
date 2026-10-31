--TEST--
dynamic construction accepts an in-scope bounded type parameter
--FILE--
<?thp

class Entity {}
class Named extends Entity {}

class Holder<U extends Entity> {
    public function __construct(U $value) {}
}

class Factory<T extends Entity> {
    public final function make(string $class, T $value): mixed {
        return new $class<T>($value);
    }
}

$factory = new Factory<Named>();
$value = $factory->make("Holder", new Named());
var_dump($value instanceof Holder);
--EXPECT--
bool(true)
