--TEST--
typed callable values capture snapshots and invoke nested closures
--FILE--
<?thp

function double(int $value): int { return $value * 2; }
function apply(callable<int, int> $callback, int $value): int {
    return $callback($value);
}

$base: int = 4;
$add = function (int $value) use ($base): int { return $value + $base; };
$base = 20;
echo apply($add, 3) . "\n";
echo apply(double, 5) . "\n";

$nested = fn(int $left): callable<int, int> => fn(int $right): int => $left + $right + $base;
$inner = $nested(2);
echo $inner(3) . "\n";

$items: vector<int> = [1];
$read = fn(): int => $items[0];
$items[0] = 9;
echo $read() . "\n";
--EXPECT--
7
10
25
1
