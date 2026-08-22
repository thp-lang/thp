--TEST--
constant collection defaults are fresh and named expressions keep source order
--FILE--
<?thp

function mark(string $value): string {
    echo "eval:" . $value . "\n";
    return $value;
}

function pair(string $first, string $second): string {
    return $first . $second;
}

function bump(vector<int> $values = [1]): int {
    $values[0] = $values[0] + 1;
    return $values[0];
}

function collect(map<string, int> $named = {"answer" => 42}, int ...$rest): int {
    return $named["answer"] + count($rest);
}

echo pair(second: mark("second"), first: mark("first")) . "\n";
echo bump() . ":" . bump() . "\n";
echo collect() . ":" . collect({"answer" => 1}, 2, 3) . "\n";
$choices: vector<int|string> = [1];
$choices[0] = "changed";
echo $choices[0] . "\n";
--EXPECT--
eval:second
eval:first
firstsecond
2:2
42:3
changed
