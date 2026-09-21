--TEST--
assigning a narrowed local invalidates the branch refinement
--FILE--
<?thp

$value: mixed = "text";
if (is_string($value)) {
    $value = 1;
    var_dump(is_int($value));
}

$nested: mixed = "text";
if (is_string($nested)) {
    if (is_string($nested)) {
        $nested = 2;
    }
    var_dump(is_int($nested));
}
--EXPECT--
bool(true)
bool(true)
