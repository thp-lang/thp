--TEST--
foundational base contracts, throwable traces, exception hierarchy, and serialization
--FILE--
<?thp
class Bag implements Countable, MapAccess<string, int> {
    private int $value = 2;
    public function count(): int { return $this->value; }
    public function offsetExists(string $offset): bool { return $offset === "value"; }
    public function offsetGet(string $offset): int { return $this->value; }
    public function offsetSet(?string $offset, int $value): void { $this->value = $value; }
    public function offsetUnset(string $offset): void { $this->value = 0; }
}
class Label {
    public function __toString(): string { return "label"; }
}
function label(Stringable $value): string { return $value->__toString(); }
function traceCount(Throwable $value): int { return count($value->getTrace()); }
function fail(): void { throw new BadMethodCallException("bad call"); }
$some = Option<?int>::some(null);
$none = Option<?int>::none();
var_dump($some->isSome());
var_dump($none->isNone());
var_dump($some->get());
$bag = new Bag();
var_dump(count($bag));
$bag->offsetSet("value", 3);
var_dump($bag->offsetGet("value"));
$bag->offsetSet(null, 4);
var_dump($bag->offsetGet("value"));
echo label(new Label()) . "\n";
try { fail(); } catch (LogicException $error) {
    var_dump($error instanceof BadFunctionCallException);
    var_dump($error->getFile() !== "");
    var_dump($error->getLine() > 0);
    var_dump(count($error->getTrace()) > 0);
    var_dump(traceCount($error) > 0);
    var_dump(count(label($error)) > 0);
}
$copy = unserialize(serialize({"numbers" => [1, 2]}));
var_dump(is_map($copy));
try { unserialize("bad"); } catch (UnexpectedValueException $error) { echo "invalid\n"; }
--EXPECT--
bool(true)
bool(true)
NULL
int(2)
int(3)
int(4)
label
bool(true)
bool(true)
bool(true)
bool(true)
bool(true)
bool(true)
bool(true)
invalid
