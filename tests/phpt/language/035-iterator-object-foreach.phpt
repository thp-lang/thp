--TEST--
foreach executes direct iterators and nested aggregates with exact protocol control flow
--FILE--
<?thp

class TraceIterator implements Iterator<int, string> {
    public vector<string> $values;
    public int $position = 0;

    public function __construct(vector<string> $values) {
        $this->values = $values;
    }

    public function rewind(): void {
        echo "rewind\n";
        $this->position = 0;
    }

    public function valid(): bool {
        echo "valid:" . $this->position . "\n";
        return $this->position < count($this->values);
    }

    public function key(): int {
        echo "key:" . $this->position . "\n";
        return $this->position;
    }

    public function value(): string {
        echo "value:" . $this->position . "\n";
        return $this->values[$this->position];
    }

    public function advance(): void {
        echo "advance:" . $this->position . "\n";
        $this->position = $this->position + 1;
    }
}

class NamedAggregate implements IteratorAggregate<int, string> {
    public string $name;
    public Traversable<int, string> $source;

    public function __construct(string $name, Traversable<int, string> $source) {
        $this->name = $name;
        $this->source = $source;
    }

    public function getIterator(): Traversable<int, string> {
        echo "aggregate:" . $this->name . "\n";
        return $this->source;
    }
}

function nested(): Traversable<int, string> {
    echo "source\n";
    return new NamedAggregate("outer", new NamedAggregate("inner", new TraceIterator(["a", "b"])));
}

foreach (nested() as $key => $value) {
    echo "body:" . $key . ":" . $value . "\n";
}

echo "value-only\n";
foreach (new TraceIterator(["only"]) as $value) {
    echo "body:" . $value . "\n";
}

$existingKey: int = 9;
$existingValue: string = "kept";
foreach (new TraceIterator([]) as $existingKey => $existingValue) {}
echo "empty:" . $existingKey . ":" . $existingValue . "\n";

echo "transfer\n";
foreach (new TraceIterator(["skip", "stop", "unused"]) as $key => $value) {
    if ($key === 0) { continue; }
    echo "kept:" . $value . "\n";
    break;
}

function first(Traversable<int, string> $items): string {
    foreach ($items as $value) { return $value; }
    return "empty";
}
echo "return:" . first(new TraceIterator(["answer"])) . "\n";

class Probe implements Closeable {
    public function close(): void { echo "close\n"; }
    public function isClosed(): bool { return false; }
}

class ThrowingIterator implements Iterator<int, string> {
    public function rewind(): void { echo "throw-rewind\n"; }
    public function valid(): bool { throw new Exception("iterator failure"); }
    public function key(): int { return 0; }
    public function value(): string { return "never"; }
    public function advance(): void {}
}

try {
    using ($probe = new Probe()) {
        try {
            foreach (new ThrowingIterator() as $value) {}
        } finally {
            echo "finally\n";
        }
    }
} catch (Exception $error) {
    echo "caught:" . $error->getMessage() . "\n";
}
--EXPECT--
source
aggregate:outer
aggregate:inner
rewind
valid:0
value:0
key:0
body:0:a
advance:0
valid:1
value:1
key:1
body:1:b
advance:1
valid:2
value-only
rewind
valid:0
value:0
body:only
advance:0
valid:1
rewind
valid:0
empty:9:kept
transfer
rewind
valid:0
value:0
key:0
advance:0
valid:1
value:1
key:1
kept:stop
rewind
valid:0
value:0
return:answer
throw-rewind
finally
close
caught:iterator failure
