--TEST--
cancelled waits leave their target running and queued cancellation skips the body
--FILE--
<?thp
scheduler_register_default();
$never = async(function (): int { echo "wrong"; return 1; });
$never->cancel();
echo $never->isStarted() . $never->isCancelled() . "\n";
try { await($never); } catch (CancellationException $error) {}
$job = async(function (): int { delay(10); echo "target"; return 7; });
try { await($job, timeout(1)); }
catch (WaitCancelledException $error) { echo "timeout\n"; }
echo await($job) . "\n";
--EXPECT--
falsetrue
timeout
target7
