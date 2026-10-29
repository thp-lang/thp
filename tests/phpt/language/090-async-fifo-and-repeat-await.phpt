--TEST--
async coroutines queue before execution and nested calls resume in FIFO order
--FILE--
<?thp
function first(): int { echo "A"; suspend(); echo "a"; return 1; }
function second(): int { echo "B"; delay(0); echo "b"; return 2; }
echo scheduler_is_registered() . "\n";
scheduler_register_default();
$one = async(function (): int { return first(); });
$two = async(function (): int { return second(); });
echo $one->isQueued() . "\n";
echo await($one) . await($two) . await($one) . "\n";
--EXPECT--
false
true
ABab121
