--TEST--
timer and delay boundaries reject invalid durations and cancelled timers settle
--FILE--
<?thp
scheduler_register_default();
try { timeout(0); } catch (Error $error) { echo "zero\n"; }
try { timeout(-1); } catch (Error $error) { echo "negative\n"; }
try { delay(-1); } catch (Error $error) { echo "delay\n"; }
$timer = timeout(100000);
$timer->cancel();
$timer->cancel();
echo $timer->isCompleted() . $timer->isCancelled() . "\n";
try { $timer->result(); }
catch (CancellationException $error) { echo "cancelled\n"; }
--EXPECT--
zero
negative
delay
truetrue
cancelled
