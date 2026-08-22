--TEST--
break and continue run cleanup when leaving a using scope
--FILE--
<?thp

class Probe implements Closeable {
    private int $id;
    private bool $closed = false;

    public function __construct(int $id) {
        $this->id = $id;
    }

    public function close(): void {
        if (!$this->closed) {
            $this->closed = true;
            echo "close:" . $this->id . "\n";
        }
    }

    public function isClosed(): bool {
        return $this->closed;
    }
}

for ($index: int = 0; $index < 3; $index = $index + 1) {
    using ($probe = new Probe($index)) {
        if ($index === 0) {
            continue;
        }
        echo "body:" . $index . "\n";
        break;
    }
}
--EXPECT--
close:0
body:1
close:1
