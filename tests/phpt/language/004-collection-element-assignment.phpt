--TEST--
collection element assignment rebuilds nested copy-on-write paths
--FILE--
<?thp

$matrix: vector<vector<int>> = [[1, 2], [3, 4]];
$copy = $matrix;
$matrix[0][1] = 9;
echo $matrix[0][1] . ":" . $copy[0][1] . "\n";

$scores: map<string, int> = {"Ada" => 10};
$scoresCopy = $scores;
$scores["Ada"] = 11;
$scores["Grace"] = 12;
echo $scores["Ada"] . ":" . $scoresCopy["Ada"] . "\n";
foreach ($scores as $name => $score) {
    echo $name . "=" . $score . "\n";
}
--EXPECT--
9:2
11:10
Ada=11
Grace=12
