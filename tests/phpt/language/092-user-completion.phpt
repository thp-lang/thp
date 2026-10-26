--TEST--
user-defined Completable notifies a pending await
--FILE--
<?thp
class Manual implements Completable<int> {
    private bool $done = false;
    private callable<void> $notify;
    public function cancel(): void {}
    public function isCompleted(): bool { return $this->done; }
    public function isCancelled(): bool { return false; }
    public function subscribe(callable<void> $notify): callable<void> {
        $this->notify = $notify;
        return function (): void {};
    }
    public function result(): int {
        if (!$this->done) { throw new Error("pending"); }
        return 42;
    }
    public function complete(): void {
        $this->done = true;
        ($this->notify)();
    }
}
scheduler_register_default();
$manual = new Manual();
$producer = async(function (Manual $manual): int {
    $manual->complete();
    return 1;
}, $manual);
echo await($manual) . await($manual) . await($producer) . "\n";
--EXPECT--
42421
