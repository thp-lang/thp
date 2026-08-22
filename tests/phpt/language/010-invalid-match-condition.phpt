--TEST--
match rejects conditions that cannot overlap the subject type
--FILE--
<?thp

$value: int = 1;
echo match ($value) {
    "one" => 1,
    default => 0,
} . "";
--EXPECTF--
%s010-invalid-match-condition.phpt:5:5: error[T0701]: match condition type `string` cannot match subject type `int`
    5 |     "one" => 1,
      |     ^^^^^
