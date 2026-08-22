--TEST--
using preserves a body exception and suppresses a cleanup exception
--FILE--
<?thp

class FailingClose implements Closeable {
    private bool $closed = false;

    public function close(): void {
        if (!$this->closed) {
            $this->closed = true;
            throw new IoException("close failed");
        }
    }

    public function isClosed(): bool {
        return $this->closed;
    }
}

try {
    using ($handle = new FailingClose()) {
        throw new Exception("body failed");
    }
} catch (Exception $error) {
    echo $error->getMessage() . "\n";
    var_dump(count($error->getSuppressed()));
    echo $error->getSuppressed()[0]->getMessage() . "\n";
}
--EXPECT--
body failed
int(1)
close failed
