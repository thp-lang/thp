--TEST--
generic classes infer constructor arguments and generic interfaces dispatch virtually
--FILE--
<?thp

interface Value<T> {
    public function value(): T;
}

class Box<T> implements Value<T> {
    public T $item;

    public function __construct(T $item) {
        $this->item = $item;
    }

    public function value(): T {
        return $this->item;
    }

    public static function make(T $item): Box<T> {
        return new Box($item);
    }
}

class GenericBase<T> {
    public static function pass(T $value): T {
        return $value;
    }

    public static function selfPass(T $value): T {
        return self::pass($value);
    }
}

class GenericChild<U> extends GenericBase<U> {
    public static function parentPass(U $value): U {
        return parent::pass($value);
    }

    public static function latePass(U $value): U {
        return static::pass($value);
    }
}

class ValueReader<T extends Value<string>> {
    public function __construct(T $source) {}

    public function read(T $source): string {
        return $source->value();
    }
}

class NamedEntity {
    public string $name;

    public function __construct(string $name) {
        $this->name = $name;
    }

    public final function label(): string {
        return $this->name;
    }
}

class NameReader<T extends NamedEntity> {
    public function __construct(T $source) {}

    public function read(T $source): string {
        return $source->name;
    }

    public function label(T $source): string {
        return $source->label();
    }
}

class SingleIterator implements Iterator<int, string> {
    public function rewind(): void {}
    public function valid(): bool { return true; }
    public function key(): int { return 0; }
    public function value(): string { return "item"; }
    public function advance(): void {}
}

class SingleAggregate implements IteratorAggregate<int, string> {
    public Traversable<int, string> $source;

    public function __construct(Traversable<int, string> $source) {
        $this->source = $source;
    }

    public function getIterator(): Traversable<int, string> {
        return $this->source;
    }
}

$inferred = new Box(42);
echo $inferred->value() . "\n";
$value: Value<string> = new Box<string>("typed");
echo $value->value() . "\n";
echo Box<int>::make(7)->value() . "\n";
echo GenericChild<string>::selfPass("self") . "\n";
echo GenericChild<string>::parentPass("parent") . "\n";
echo GenericChild<string>::latePass("static") . "\n";
if ($inferred instanceof Box) {
    echo "erased\n";
}
$reader = new ValueReader($value);
echo $reader->read($value) . "\n";
$nameReader = new NameReader(new NamedEntity("bound"));
echo $nameReader->read(new NamedEntity("property")) . "\n";
echo $nameReader->label(new NamedEntity("final")) . "\n";
$cursor: Iterator<int, string> = new SingleIterator();
$cursor->rewind();
echo $cursor->key() . ":" . $cursor->value() . "\n";
$aggregate: IteratorAggregate<int, string> = new SingleAggregate($cursor);
$traversable: Traversable<int, string> = $aggregate->getIterator();
if ($traversable instanceof SingleIterator) {
    echo "aggregate\n";
}
--EXPECT--
42
typed
7
self
parent
static
erased
typed
property
final
0:item
aggregate
