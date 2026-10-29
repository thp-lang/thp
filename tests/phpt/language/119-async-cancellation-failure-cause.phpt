--TEST--
failed cancellation source becomes the wait failure cause and target keeps running
--FILE--
<?thp
scheduler_register_default();
$target = async(function (): int { delay(2); return 7; });
$cancellation = async(function (): int {
    throw new RuntimeException("stop");
});
try { await($target, $cancellation); }
catch (WaitCancelledException $error) {
    $cause = $error->getPrevious();
    if ($cause instanceof RuntimeException) { echo $cause->getMessage() . "\n"; }
}
echo await($target) . "\n";
$nestedTarget = async(function (): int { delay(2); return 9; });
$nestedCancellation = async(function (): int {
    throw new RuntimeException("nested");
});
$waiter = async(function () use ($nestedTarget, $nestedCancellation): void {
    try { await($nestedTarget, $nestedCancellation); }
    catch (WaitCancelledException $error) {
        $cause = $error->getPrevious();
        if ($cause instanceof RuntimeException) { echo $cause->getMessage() . "\n"; }
    }
});
await($waiter);
echo await($nestedTarget) . "\n";
--EXPECT--
stop
7
nested
9
