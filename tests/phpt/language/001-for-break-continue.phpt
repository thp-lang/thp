--TEST--
for clauses, break, and continue preserve source-order control flow
--FILE--
<?thp

$sum: int = 0;
for ($left: int = 0, $right: int = 5; $left < 100, $left < $right; $left = $left + 1, $right = $right - 1) {
    if ($left === 1) {
        continue;
    }
    $sum = $sum + $left;
}

for (;;) {
    $sum = $sum + 10;
    break;
}

echo $sum . "\n";
--EXPECT--
12
