--TEST--
deadlock is catchable and completion subscriptions notify once or unsubscribe
--FILE--
<?thp
class Stalled implements Completable<int> {
    public function cancel(): void {}
    public function isCompleted(): bool { return false; }
    public function isCancelled(): bool { return false; }
    public function subscribe(callable<void> $notify): callable<void> {
        return function (): void {};
    }
    public function result(): int { throw new Error("pending"); }
}
scheduler_register_default();
try { await(new Stalled()); }
catch (AsyncDeadlockException $error) { echo "deadlock\n"; }
$job = async(fn(): int => 7);
$unsubscribe = $job->subscribe(function (): void { echo "wrong\n"; });
$unsubscribe();
$unsubscribe();
echo await($job) . "\n";
$job->subscribe(function (): void { echo "complete\n"; });
--EXPECT--
deadlock
7
complete
