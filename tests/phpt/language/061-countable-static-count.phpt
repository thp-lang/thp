--TEST--
unrelated static count methods do not enforce the Countable result contract
--FILE--
<?thp
class Counter implements Countable {
    public function count(): int { return 1; }
}
class Calculator {
    public static function count(Countable $value): int { return -1; }
}
echo Calculator::count(new Counter());
--EXPECT--
-1
