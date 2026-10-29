--TEST--
collection callbacks suspend and preserve vector and map results
--FILE--
<?thp
scheduler_register_default();
$job = async(function (): void {
    $mapped = vector_map([1, 2], function (int $value): int {
        suspend();
        return $value + 1;
    });
    echo $mapped[0] . $mapped[1];
    $filtered = vector_filter([1, 2], function (int $value): bool {
        suspend();
        return $value == 2;
    });
    echo $filtered[0];
    $changed = map_transform({"a" => 1}, function (int $value, string $key): int {
        suspend();
        return $value + 3;
    });
    echo $changed["a"];
    $kept = map_filter({"a" => 1, "b" => 2}, function (int $value, string $key): bool {
        suspend();
        return $key == "b";
    });
    echo $kept["b"];
});
$other = async(function (): void { echo "B"; });
echo "A";
await($job);
await($other);
--EXPECT--
AB23242
