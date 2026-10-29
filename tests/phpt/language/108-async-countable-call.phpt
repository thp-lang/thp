--TEST--
count on a user Countable suspends through a coroutine frame
--FILE--
<?thp
class SlowCount implements Countable {
    public function count(): int {
        echo "A";
        suspend();
        echo "C";
        return 2;
    }
}
scheduler_register_default();
$reader = async(function (): void { echo count(new SlowCount()); });
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABC2
