--TEST--
foreach traverses vectors and insertion-ordered maps with typed keys
--FILE--
<?thp

$values: vector<int> = [10, 20, 30];
$last: int = 0;
foreach ($values as $index => $last) {
    if ($index === 1) {
        continue;
    }
    echo $index . ":" . $last . "\n";
}
echo "last:" . $last . "\n";

$scores: map<string, int> = {"Ada" => 10, "Grace" => 20};
foreach ($scores as $name => $score) {
    echo $name . ":" . $score . "\n";
}
--EXPECT--
0:10
2:30
last:30
Ada:10
Grace:20
