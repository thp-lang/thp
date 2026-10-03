--TEST--
collection callbacks preserve value order and map key positions
--FILE--
<?thp

$values: vector<int> = [1, 2, 3, 4];
$mapped = vector_map($values, fn(int $value): int => $value * 2);
foreach ($mapped as $key => $value) { echo $key . ":" . $value . "\n"; }
$filtered = vector_filter($values, fn(int $value): bool => $value % 2 == 0);
foreach ($filtered as $key => $value) { echo "f" . $key . ":" . $value . "\n"; }
$slice = vector_slice($values, -3, -1);
foreach ($slice as $value) { echo "s" . $value . "\n"; }
echo count(vector_concat($filtered, $slice)) . "\n";

$scores: map<string, int> = {"a" => 1, "b" => 2};
$changed = map_transform($scores, fn(int $value, string $key): int => $value + 10);
foreach ($changed as $key => $value) { echo "t" . $key . ":" . $value . "\n"; }
$kept = map_filter($scores, fn(int $value, string $key): bool => $key == "b");
foreach ($kept as $key => $value) { echo "k" . $key . ":" . $value . "\n"; }
$merged = map_merge($scores, {"b" => 9, "c" => 3});
foreach ($merged as $key => $value) { echo "m" . $key . ":" . $value . "\n"; }
echo count(vector_map([], fn(int $value): string => "x")) . "\n";
$empty: vector<int> = vector_slice([], 0);
echo count($empty) . "\n";
$emptyMap = map_filter({}, fn(int $value, string $key): bool => true);
echo count($emptyMap) . "\n";
--EXPECT--
0:2
1:4
2:6
3:8
f0:2
f1:4
s2
s3
4
ta:11
tb:12
kb:2
ma:1
mb:9
mc:3
0
0
0
