--TEST--
count reads native lengths without consuming traversal and foreach keeps its COW snapshot
--FILE--
<?thp

$bytes: string = "A\x00B";
$values: vector<int> = [10, 20];
$scores: map<string, int> = {"Ada" => 1, "Grace" => 2};

echo count($bytes) . ":" . count($values) . ":" . count($scores) . "\n";

foreach ($values as $index => $value) {
    echo $value . "\n";
    if ($index === 0) {
        $values[1] = 99;
    }
}

echo count($values) . ":" . $values[1] . "\n";
foreach ($values as $value) {
    echo $value . "\n";
}
--EXPECT--
3:2:2
10
20
2:99
10
99
