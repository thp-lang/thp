--TEST--
match evaluates lazily in source order and throws a catchable unmatched error
--FILE--
<?thp

function probe(int $value): int {
    echo "probe:" . $value . "\n";
    return $value;
}

$subject: int = 2;
$result = match ($subject) {
    probe(1), probe(2) => "selected",
    probe(3) => "late",
    default => "fallback",
};
echo $result . "\n";

$mixed = match (1) {
    1 => 42,
    default => "other",
};
echo $mixed . "\n";

try {
    echo match (9) {
        1 => "one",
    };
} catch (UnhandledMatchError $error) {
    echo $error->getMessage() . "\n";
}
--EXPECT--
probe:1
probe:2
selected
42
no match arm handled int 9
