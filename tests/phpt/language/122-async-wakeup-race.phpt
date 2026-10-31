--TEST--
first processed completion wins a two-source wait and releases the loser
--FILE--
<?thp
class ManualResult implements Completable<int> {
    private bool $done = false;
    private bool $unsubscribed = false;
    private int $value;
    private callable<void> $notify;
    public function __construct(int $value) { $this->value = $value; }
    public function cancel(): void {}
    public function isCompleted(): bool { return $this->done; }
    public function isCancelled(): bool { return false; }
    public function subscribe(callable<void> $notify): callable<void> {
        $this->notify = $notify;
        if ($this->done) { $notify(); }
        $self = $this;
        return function () use ($self): void { $self->markUnsubscribed(); };
    }
    public function markUnsubscribed(): void { $this->unsubscribed = true; }
    public function wasUnsubscribed(): bool { return $this->unsubscribed; }
    public function result(): int {
        if (!$this->done) { throw new Error("pending"); }
        return $this->value;
    }
    public function complete(): void {
        $this->done = true;
        ($this->notify)();
    }
}
scheduler_register_default();
$target = new ManualResult(7);
$cancellation = new ManualResult(8);
$waiter = async(function () use ($target, $cancellation): void {
    echo await($target, $cancellation);
});
$producer = async(function () use ($target, $cancellation): void {
    $target->complete();
    $cancellation->complete();
});
await($waiter);
await($producer);
echo $cancellation->wasUnsubscribed() . "\n";
$target2 = new ManualResult(9);
$cancellation2 = new ManualResult(10);
$waiter2 = async(function () use ($target2, $cancellation2): void {
    try { await($target2, $cancellation2); }
    catch (WaitCancelledException $error) { echo "cancel"; }
});
$producer2 = async(function () use ($target2, $cancellation2): void {
    $cancellation2->complete();
    $target2->complete();
});
await($waiter2);
await($producer2);
echo $target2->wasUnsubscribed() . "\n";
--EXPECT--
7true
canceltrue
