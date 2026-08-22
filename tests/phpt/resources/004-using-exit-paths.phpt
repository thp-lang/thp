--TEST--
using closes exactly once on fallthrough, return, and exception
--FILE--
<?thp

class Probe implements Closeable {
    private bool $closed = false;
    private string $name;

    public function __construct(string $name) {
        $this->name = $name;
    }

    public function close(): void {
        if (!$this->closed) {
            $this->closed = true;
            echo "close:" . $this->name . "\n";
        }
    }

    public function isClosed(): bool {
        return $this->closed;
    }
}

function returnFromUsing(): string {
    using ($probe = new Probe("return")) {
        return "returned";
    }
}

using ($probe = new Probe("fallthrough")) {
    echo "body\n";
}

echo returnFromUsing() . "\n";

try {
    using ($probe = new Probe("exception")) {
        throw new Exception("body failed");
    }
} catch (Exception $error) {
    echo $error->getMessage() . "\n";
}
--EXPECT--
body
close:fallthrough
close:return
returned
close:exception
body failed
